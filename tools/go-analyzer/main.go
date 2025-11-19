package main

import (
	"encoding/json"
	"fmt"
	"go/ast"
	"go/parser"
	"go/token"
	"os"
	"path/filepath"
	"strings"
)

// ReachabilityReport matches the format expected by Rust
type ReachabilityReport struct {
	AllFunctions         map[string]FunctionNode `json:"all_functions"`
	ReachableFunctions   []string                `json:"reachable_functions"`
	UnreachableFunctions []string                `json:"unreachable_functions"`
	Entrypoints          []string                `json:"entrypoints"`
}

// FunctionNode represents a function in the call graph
type FunctionNode struct {
	ID          string   `json:"id"`
	Name        string   `json:"name"`
	File        string   `json:"file"`
	Line        int      `json:"line"`
	IsEntrypoint bool    `json:"is_entrypoint"`
	Reachable   bool     `json:"reachable"`
	Calls       []string `json:"calls"`
}

func main() {
	if len(os.Args) < 2 {
		fmt.Fprintf(os.Stderr, "Usage: go-analyzer <project-root>\n")
		os.Exit(1)
	}

	projectRoot := os.Args[1]
	
	report, err := analyzeGoProject(projectRoot)
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error: %v\n", err)
		os.Exit(1)
	}

	// Output JSON to stdout
	enc := json.NewEncoder(os.Stdout)
	enc.SetIndent("", "  ")
	if err := enc.Encode(report); err != nil {
		fmt.Fprintf(os.Stderr, "Error encoding JSON: %v\n", err)
		os.Exit(1)
	}
}

func analyzeGoProject(root string) (*ReachabilityReport, error) {
	fset := token.NewFileSet()
	functions := make(map[string]FunctionNode)
	entrypoints := []string{}

	// Walk the project and parse all .go files
	err := filepath.Walk(root, func(path string, info os.FileInfo, err error) error {
		if err != nil {
			return err
		}

		// Skip vendor, .git, etc.
		if info.IsDir() && (info.Name() == "vendor" || info.Name() == ".git" || info.Name() == "node_modules") {
			return filepath.SkipDir
		}

		if !info.IsDir() && strings.HasSuffix(path, ".go") && !strings.HasSuffix(path, "_test.go") {
			return parseGoFile(path, fset, functions, &entrypoints)
		}

		return nil
	})

	if err != nil {
		return nil, err
	}

	// Perform reachability analysis from entrypoints
	reachable := make(map[string]bool)
	for _, ep := range entrypoints {
		markReachable(ep, functions, reachable)
	}

	// Update reachability status
	for id := range functions {
		fn := functions[id]
		fn.Reachable = reachable[id]
		functions[id] = fn
	}

	// Build lists
	reachableFuncs := []string{}
	unreachableFuncs := []string{}
	for id, fn := range functions {
		if fn.Reachable {
			reachableFuncs = append(reachableFuncs, id)
		} else {
			unreachableFuncs = append(unreachableFuncs, id)
		}
	}

	return &ReachabilityReport{
		AllFunctions:         functions,
		ReachableFunctions:   reachableFuncs,
		UnreachableFunctions: unreachableFuncs,
		Entrypoints:          entrypoints,
	}, nil
}

func parseGoFile(path string, fset *token.FileSet, functions map[string]FunctionNode, entrypoints *[]string) error {
	file, err := parser.ParseFile(fset, path, nil, parser.ParseComments)
	if err != nil {
		return err
	}

	// Extract package name
	pkgName := file.Name.Name

	// Visit all function declarations
	ast.Inspect(file, func(n ast.Node) bool {
		funcDecl, ok := n.(*ast.FuncDecl)
		if !ok {
			return true
		}

		funcName := funcDecl.Name.Name
		funcID := fmt.Sprintf("%s.%s", pkgName, funcName)
		
		// Check if it's main() or init()
		isEntry := funcName == "main" || funcName == "init"
		if isEntry {
			*entrypoints = append(*entrypoints, funcID)
		}

		// Extract function calls
		calls := []string{}
		ast.Inspect(funcDecl.Body, func(n ast.Node) bool {
			callExpr, ok := n.(*ast.CallExpr)
			if !ok {
				return true
			}

			// Try to resolve the called function
			if ident, ok := callExpr.Fun.(*ast.Ident); ok {
				calls = append(calls, fmt.Sprintf("%s.%s", pkgName, ident.Name))
			} else if sel, ok := callExpr.Fun.(*ast.SelectorExpr); ok {
				if pkgIdent, ok := sel.X.(*ast.Ident); ok {
					calls = append(calls, fmt.Sprintf("%s.%s", pkgIdent.Name, sel.Sel.Name))
				}
			}

			return true
		})

		functions[funcID] = FunctionNode{
			ID:          funcID,
			Name:        funcName,
			File:        path,
			Line:        fset.Position(funcDecl.Pos()).Line,
			IsEntrypoint: isEntry,
			Reachable:   false,
			Calls:       calls,
		}

		return false
	})

	return nil
}

func markReachable(funcID string, functions map[string]FunctionNode, reachable map[string]bool) {
	if reachable[funcID] {
		return // Already visited
	}

	reachable[funcID] = true

	fn, exists := functions[funcID]
	if !exists {
		return
	}

	// Recursively mark called functions as reachable
	for _, callee := range fn.Calls {
		markReachable(callee, functions, reachable)
	}
}
