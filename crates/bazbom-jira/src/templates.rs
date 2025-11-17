use crate::error::{JiraError, Result};
use crate::models::*;
use regex::Regex;
use std::collections::HashMap;

/// Template engine for Jira ticket generation
pub struct TemplateEngine {
    /// Title template
    title_template: String,

    /// Description template
    description_template: String,
}

impl TemplateEngine {
    /// Create a new template engine with default templates
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a template engine with custom templates
    pub fn with_templates(title_template: String, description_template: String) -> Self {
        Self {
            title_template,
            description_template,
        }
    }

    /// Render a ticket title with variable substitution
    pub fn render_title(&self, variables: &HashMap<String, String>) -> Result<String> {
        let mut result = self.title_template.clone();

        // Replace all {variable} placeholders with values from the map
        for (key, value) in variables {
            let placeholder = format!("{{{}}}", key);
            result = result.replace(&placeholder, value);
        }

        Ok(result)
    }

    /// Render a ticket description with Markdown to ADF conversion
    pub fn render_description(
        &self,
        variables: &HashMap<String, String>,
    ) -> Result<JiraDescription> {
        // First, substitute variables in the template
        let mut markdown = self.description_template.clone();
        for (key, value) in variables {
            let placeholder = format!("{{{}}}", key);
            markdown = markdown.replace(&placeholder, value);
        }

        // Convert Markdown to Jira ADF
        let content = self.markdown_to_adf(&markdown)?;

        Ok(JiraDescription {
            doc_type: "doc".to_string(),
            version: 1,
            content,
        })
    }

    /// Convert Markdown text to Jira ADF content blocks
    fn markdown_to_adf(&self, markdown: &str) -> Result<Vec<JiraContent>> {
        let mut content = Vec::new();
        let lines: Vec<&str> = markdown.lines().collect();
        let mut i = 0;

        while i < lines.len() {
            let line = lines[i].trim();

            // Skip empty lines
            if line.is_empty() {
                i += 1;
                continue;
            }

            // Heading (h1-h6 or ## style)
            if let Some(heading) = self.parse_heading(line)? {
                content.push(heading);
                i += 1;
                continue;
            }

            // Code block (```lang or {code:lang})
            if line.starts_with("```") || line.starts_with("{code") {
                let (code_block, lines_consumed) = self.parse_code_block(&lines[i..])?;
                content.push(code_block);
                i += lines_consumed;
                continue;
            }

            // Bullet list (* or •)
            if line.starts_with("* ") || line.starts_with("• ") || line.starts_with("- ") {
                let (bullet_list, lines_consumed) = self.parse_bullet_list(&lines[i..])?;
                content.push(bullet_list);
                i += lines_consumed;
                continue;
            }

            // Default: paragraph
            let paragraph = self.parse_paragraph(line)?;
            content.push(paragraph);
            i += 1;
        }

        Ok(content)
    }

    /// Parse a heading line
    fn parse_heading(&self, line: &str) -> Result<Option<JiraContent>> {
        // Jira Wiki style: h1. Text
        let re_wiki = Regex::new(r"^h([1-6])\.\s+(.+)$")
            .map_err(|e| JiraError::Template(format!("Regex error: {}", e)))?;

        if let Some(caps) = re_wiki.captures(line) {
            let level = caps[1]
                .parse::<i32>()
                .map_err(|e| JiraError::Template(format!("Invalid heading level: {}", e)))?;
            let text = &caps[2];

            return Ok(Some(JiraContent::Heading {
                attrs: HeadingAttrs { level },
                content: self.parse_inline_text(text)?,
            }));
        }

        // Markdown style: ## Text
        let re_md = Regex::new(r"^(#{1,6})\s+(.+)$")
            .map_err(|e| JiraError::Template(format!("Regex error: {}", e)))?;

        if let Some(caps) = re_md.captures(line) {
            let level = caps[1].len() as i32;
            let text = &caps[2];

            return Ok(Some(JiraContent::Heading {
                attrs: HeadingAttrs { level },
                content: self.parse_inline_text(text)?,
            }));
        }

        Ok(None)
    }

    /// Parse a code block
    fn parse_code_block(&self, lines: &[&str]) -> Result<(JiraContent, usize)> {
        let first_line = lines[0];
        let mut language = None;
        let mut code_lines = Vec::new();
        let mut i = 1;

        // Detect format and language
        if first_line.starts_with("```") {
            // Markdown style: ```lang
            language = first_line
                .trim_start_matches("```")
                .trim()
                .to_string()
                .into();

            // Find closing ```
            while i < lines.len() {
                if lines[i].trim() == "```" {
                    i += 1;
                    break;
                }
                code_lines.push(lines[i]);
                i += 1;
            }
        } else if first_line.starts_with("{code") {
            // Jira Wiki style: {code:lang}
            let re = Regex::new(r"\{code:?([^}]*)\}")
                .map_err(|e| JiraError::Template(format!("Regex error: {}", e)))?;

            if let Some(caps) = re.captures(first_line) {
                let lang = caps[1].trim();
                if !lang.is_empty() {
                    language = Some(lang.to_string());
                }
            }

            // Find closing {code}
            while i < lines.len() {
                if lines[i].trim() == "{code}" {
                    i += 1;
                    break;
                }
                code_lines.push(lines[i]);
                i += 1;
            }
        }

        let code_text = code_lines.join("\n");

        Ok((
            JiraContent::CodeBlock {
                attrs: CodeBlockAttrs { language },
                content: vec![JiraTextNode {
                    node_type: "text".to_string(),
                    text: code_text,
                    marks: None,
                }],
            },
            i,
        ))
    }

    /// Parse a bullet list
    fn parse_bullet_list(&self, lines: &[&str]) -> Result<(JiraContent, usize)> {
        let mut items = Vec::new();
        let mut i = 0;

        while i < lines.len() {
            let line = lines[i].trim();

            // Check if this is a list item
            if line.starts_with("* ") || line.starts_with("• ") || line.starts_with("- ") {
                let text = line
                    .trim_start_matches("* ")
                    .trim_start_matches("• ")
                    .trim_start_matches("- ");

                items.push(ListItem {
                    item_type: "listItem".to_string(),
                    content: vec![JiraContent::Paragraph {
                        content: self.parse_inline_text(text)?,
                    }],
                });

                i += 1;
            } else {
                // Not a list item, stop parsing
                break;
            }
        }

        Ok((JiraContent::BulletList { content: items }, i))
    }

    /// Parse a paragraph
    fn parse_paragraph(&self, line: &str) -> Result<JiraContent> {
        Ok(JiraContent::Paragraph {
            content: self.parse_inline_text(line)?,
        })
    }

    /// Parse inline text with formatting (bold, italic, code, links)
    fn parse_inline_text(&self, text: &str) -> Result<Vec<JiraTextNode>> {
        let mut nodes = Vec::new();
        let mut current_text = String::new();
        let mut chars = text.chars().peekable();

        while let Some(ch) = chars.next() {
            match ch {
                '*' => {
                    // Check for bold (**text** or *text*)
                    if chars.peek() == Some(&'*') {
                        // Bold: **text**
                        chars.next(); // consume second *

                        // Flush current text
                        if !current_text.is_empty() {
                            nodes.push(JiraTextNode {
                                node_type: "text".to_string(),
                                text: current_text.clone(),
                                marks: None,
                            });
                            current_text.clear();
                        }

                        // Find closing **
                        let mut bold_text = String::new();
                        let mut found_close = false;
                        while let Some(ch) = chars.next() {
                            if ch == '*' && chars.peek() == Some(&'*') {
                                chars.next(); // consume second *
                                found_close = true;
                                break;
                            }
                            bold_text.push(ch);
                        }

                        if found_close {
                            nodes.push(JiraTextNode {
                                node_type: "text".to_string(),
                                text: bold_text,
                                marks: Some(vec![TextMark {
                                    mark_type: "strong".to_string(),
                                    attrs: None,
                                }]),
                            });
                        } else {
                            // Not properly closed, treat as literal
                            current_text.push_str("**");
                            current_text.push_str(&bold_text);
                        }
                    } else {
                        // Italic: *text*
                        // Flush current text
                        if !current_text.is_empty() {
                            nodes.push(JiraTextNode {
                                node_type: "text".to_string(),
                                text: current_text.clone(),
                                marks: None,
                            });
                            current_text.clear();
                        }

                        // Find closing *
                        let mut italic_text = String::new();
                        let mut found_close = false;
                        for ch in chars.by_ref() {
                            if ch == '*' {
                                found_close = true;
                                break;
                            }
                            italic_text.push(ch);
                        }

                        if found_close && !italic_text.is_empty() {
                            nodes.push(JiraTextNode {
                                node_type: "text".to_string(),
                                text: italic_text,
                                marks: Some(vec![TextMark {
                                    mark_type: "em".to_string(),
                                    attrs: None,
                                }]),
                            });
                        } else {
                            // Not properly closed, treat as literal
                            current_text.push('*');
                            current_text.push_str(&italic_text);
                        }
                    }
                }
                '`' => {
                    // Inline code: `text`
                    // Flush current text
                    if !current_text.is_empty() {
                        nodes.push(JiraTextNode {
                            node_type: "text".to_string(),
                            text: current_text.clone(),
                            marks: None,
                        });
                        current_text.clear();
                    }

                    // Find closing `
                    let mut code_text = String::new();
                    let mut found_close = false;
                    for ch in chars.by_ref() {
                        if ch == '`' {
                            found_close = true;
                            break;
                        }
                        code_text.push(ch);
                    }

                    if found_close {
                        nodes.push(JiraTextNode {
                            node_type: "text".to_string(),
                            text: code_text,
                            marks: Some(vec![TextMark {
                                mark_type: "code".to_string(),
                                attrs: None,
                            }]),
                        });
                    } else {
                        // Not properly closed, treat as literal
                        current_text.push('`');
                        current_text.push_str(&code_text);
                    }
                }
                _ => {
                    current_text.push(ch);
                }
            }
        }

        // Flush remaining text
        if !current_text.is_empty() {
            nodes.push(JiraTextNode {
                node_type: "text".to_string(),
                text: current_text,
                marks: None,
            });
        }

        // Return at least one node (empty text if needed)
        if nodes.is_empty() {
            nodes.push(JiraTextNode {
                node_type: "text".to_string(),
                text: String::new(),
                marks: None,
            });
        }

        Ok(nodes)
    }
}

impl Default for TemplateEngine {
    fn default() -> Self {
        Self {
            title_template: "[SECURITY] {cve_id} in {package} {version} ({severity})".to_string(),
            description_template: r#"h2. 🎯 Summary
**Why Fix This:** {why_fix}

**CVE:** {cve_id}
**Package:** {package}
**Current Version:** {current_version}
**Fix Version:** {fix_version}
**Severity:** {severity} (CVSS {cvss_score})
**Priority:** {priority}
**Reachability:** {reachability_status}

h2. 🚨 Vulnerability Details
**Impact:** {impact_description}

**EPSS Score:** {epss_score}
**KEV Status:** {kev_status}
**Exploit Intelligence:**
{exploit_intel}

h2. 📊 ML Risk Score
**Overall Risk:** {ml_risk_score}/100 ({ml_risk_level})

{ml_risk_breakdown}

h2. 🔍 Reachability Analysis
**Status:** {reachability_status} ({reachability_confidence}% confidence)

**Call Paths ({call_path_count}):**
```
{call_paths}
```

**Files Affected:**
{affected_files}

h2. 📏 Difficulty Scoring
**Remediation Difficulty:** {remediation_difficulty}/100 ({remediation_level})
**Estimated Time:** {remediation_time}

**Why {remediation_level}:**
{remediation_reasons}

h2. 🔧 Remediation
**Fix:** {fix_description}

**Breaking Changes:** {breaking_changes_status}
{breaking_changes_details}

**Fix Command:**
```bash
{fix_command}
```

h2. 🎓 Framework Guidance
**Framework:** {framework_name}
**Migration:** {migration_required}

**Compatible With:**
{compatibility_info}

h2. 🧪 Testing Strategy
**Recommended Tests:**
{test_recommendations}

h2. 📦 Container Impact
**Affected Images:** {container_image_count}
{container_details}

h2. 🛡 Policy Compliance
**Before:** {policy_violations_before}
**After:** {policy_violations_after}

**Frameworks:**
{compliance_frameworks}

h2. 🔗 Links
• [BazBOM Scan|{bazbom_link}]
• [CVE Details|{cve_link}]
• [GitHub PR|{github_pr_link}]
{additional_links}

**Attachments:**
{attachments_list}
"#
            .to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_engine_creation() {
        let engine = TemplateEngine::new();
        assert!(engine.title_template.contains("{cve_id}"));
    }

    #[test]
    fn test_custom_template_engine() {
        let engine = TemplateEngine::with_templates(
            "Custom title: {var}".to_string(),
            "Custom description: {var}".to_string(),
        );
        assert_eq!(engine.title_template, "Custom title: {var}");
        assert_eq!(engine.description_template, "Custom description: {var}");
    }

    #[test]
    fn test_render_title_simple() {
        let engine = TemplateEngine::with_templates(
            "[SECURITY] {cve_id} in {package}".to_string(),
            "desc".to_string(),
        );

        let mut variables = HashMap::new();
        variables.insert("cve_id".to_string(), "CVE-2024-1234".to_string());
        variables.insert("package".to_string(), "log4j-core".to_string());

        let result = engine.render_title(&variables).unwrap();
        assert_eq!(result, "[SECURITY] CVE-2024-1234 in log4j-core");
    }

    #[test]
    fn test_render_title_multiple_variables() {
        let engine = TemplateEngine::with_templates(
            "{severity} - {cve_id} - {version}".to_string(),
            "desc".to_string(),
        );

        let mut variables = HashMap::new();
        variables.insert("severity".to_string(), "CRITICAL".to_string());
        variables.insert("cve_id".to_string(), "CVE-2024-5678".to_string());
        variables.insert("version".to_string(), "2.17.0".to_string());

        let result = engine.render_title(&variables).unwrap();
        assert_eq!(result, "CRITICAL - CVE-2024-5678 - 2.17.0");
    }

    #[test]
    fn test_parse_heading_jira_wiki_style() {
        let engine = TemplateEngine::new();
        let heading = engine.parse_heading("h2. Test Heading").unwrap();

        assert!(heading.is_some());
        match heading.unwrap() {
            JiraContent::Heading { attrs, content } => {
                assert_eq!(attrs.level, 2);
                assert_eq!(content.len(), 1);
                assert_eq!(content[0].text, "Test Heading");
            }
            _ => panic!("Expected Heading variant"),
        }
    }

    #[test]
    fn test_parse_heading_markdown_style() {
        let engine = TemplateEngine::new();
        let heading = engine.parse_heading("## Test Heading").unwrap();

        assert!(heading.is_some());
        match heading.unwrap() {
            JiraContent::Heading { attrs, content } => {
                assert_eq!(attrs.level, 2);
                assert_eq!(content.len(), 1);
                assert_eq!(content[0].text, "Test Heading");
            }
            _ => panic!("Expected Heading variant"),
        }
    }

    #[test]
    fn test_parse_heading_all_levels() {
        let engine = TemplateEngine::new();

        for level in 1..=6 {
            let heading_md = format!("{} Test", "#".repeat(level));
            let result = engine.parse_heading(&heading_md).unwrap();
            assert!(result.is_some());

            let heading_wiki = format!("h{}. Test", level);
            let result = engine.parse_heading(&heading_wiki).unwrap();
            assert!(result.is_some());
        }
    }

    #[test]
    fn test_parse_inline_text_plain() {
        let engine = TemplateEngine::new();
        let nodes = engine.parse_inline_text("Plain text").unwrap();

        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].text, "Plain text");
        assert!(nodes[0].marks.is_none());
    }

    #[test]
    fn test_parse_inline_text_bold() {
        let engine = TemplateEngine::new();
        let nodes = engine.parse_inline_text("**bold text**").unwrap();

        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].text, "bold text");
        assert!(nodes[0].marks.is_some());

        let marks = nodes[0].marks.as_ref().unwrap();
        assert_eq!(marks.len(), 1);
        assert_eq!(marks[0].mark_type, "strong");
    }

    #[test]
    fn test_parse_inline_text_italic() {
        let engine = TemplateEngine::new();
        let nodes = engine.parse_inline_text("*italic text*").unwrap();

        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].text, "italic text");
        assert!(nodes[0].marks.is_some());

        let marks = nodes[0].marks.as_ref().unwrap();
        assert_eq!(marks.len(), 1);
        assert_eq!(marks[0].mark_type, "em");
    }

    #[test]
    fn test_parse_inline_text_code() {
        let engine = TemplateEngine::new();
        let nodes = engine.parse_inline_text("`inline code`").unwrap();

        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].text, "inline code");
        assert!(nodes[0].marks.is_some());

        let marks = nodes[0].marks.as_ref().unwrap();
        assert_eq!(marks.len(), 1);
        assert_eq!(marks[0].mark_type, "code");
    }

    #[test]
    fn test_parse_inline_text_mixed() {
        let engine = TemplateEngine::new();
        let nodes = engine
            .parse_inline_text("Plain **bold** and *italic* and `code`")
            .unwrap();

        // Should produce 6 nodes:
        // 1. "Plain " (no marks)
        // 2. "bold" (strong mark)
        // 3. " and " (no marks)
        // 4. "italic" (em mark)
        // 5. " and " (no marks)
        // 6. "code" (code mark)
        assert_eq!(nodes.len(), 6);

        // First node: "Plain "
        assert_eq!(nodes[0].text, "Plain ");
        assert!(nodes[0].marks.is_none());

        // Second node: "bold"
        assert_eq!(nodes[1].text, "bold");
        assert_eq!(nodes[1].marks.as_ref().unwrap()[0].mark_type, "strong");

        // Third node: " and "
        assert_eq!(nodes[2].text, " and ");
        assert!(nodes[2].marks.is_none());

        // Fourth node: "italic"
        assert_eq!(nodes[3].text, "italic");
        assert_eq!(nodes[3].marks.as_ref().unwrap()[0].mark_type, "em");

        // Fifth node: " and "
        assert_eq!(nodes[4].text, " and ");
        assert!(nodes[4].marks.is_none());

        // Sixth node: "code"
        assert_eq!(nodes[5].text, "code");
        assert_eq!(nodes[5].marks.as_ref().unwrap()[0].mark_type, "code");
    }

    #[test]
    fn test_parse_paragraph() {
        let engine = TemplateEngine::new();
        let paragraph = engine.parse_paragraph("Test paragraph text").unwrap();

        match paragraph {
            JiraContent::Paragraph { content } => {
                assert_eq!(content.len(), 1);
                assert_eq!(content[0].text, "Test paragraph text");
            }
            _ => panic!("Expected Paragraph variant"),
        }
    }

    #[test]
    fn test_parse_code_block_markdown() {
        let engine = TemplateEngine::new();
        let lines = vec!["```bash", "echo 'hello'", "ls -la", "```"];
        let (code_block, consumed) = engine.parse_code_block(&lines).unwrap();

        assert_eq!(consumed, 4);
        match code_block {
            JiraContent::CodeBlock { attrs, content } => {
                assert_eq!(attrs.language, Some("bash".to_string()));
                assert_eq!(content.len(), 1);
                assert_eq!(content[0].text, "echo 'hello'\nls -la");
            }
            _ => panic!("Expected CodeBlock variant"),
        }
    }

    #[test]
    fn test_parse_code_block_jira_wiki() {
        let engine = TemplateEngine::new();
        let lines = vec!["{code:java}", "System.out.println(\"test\");", "{code}"];
        let (code_block, consumed) = engine.parse_code_block(&lines).unwrap();

        assert_eq!(consumed, 3);
        match code_block {
            JiraContent::CodeBlock { attrs, content } => {
                assert_eq!(attrs.language, Some("java".to_string()));
                assert_eq!(content.len(), 1);
                assert_eq!(content[0].text, "System.out.println(\"test\");");
            }
            _ => panic!("Expected CodeBlock variant"),
        }
    }

    #[test]
    fn test_parse_bullet_list() {
        let engine = TemplateEngine::new();
        let lines = vec!["* Item 1", "* Item 2", "* Item 3", ""];
        let (bullet_list, consumed) = engine.parse_bullet_list(&lines).unwrap();

        assert_eq!(consumed, 3);
        match bullet_list {
            JiraContent::BulletList { content } => {
                assert_eq!(content.len(), 3);
                // Check first item
                assert_eq!(content[0].item_type, "listItem");
            }
            _ => panic!("Expected BulletList variant"),
        }
    }

    #[test]
    fn test_parse_bullet_list_different_bullets() {
        let engine = TemplateEngine::new();

        // Test with *
        let lines_star = vec!["* Item 1", "* Item 2"];
        let (list_star, _) = engine.parse_bullet_list(&lines_star).unwrap();
        match list_star {
            JiraContent::BulletList { content } => assert_eq!(content.len(), 2),
            _ => panic!("Expected BulletList"),
        }

        // Test with •
        let lines_bullet = vec!["• Item 1", "• Item 2"];
        let (list_bullet, _) = engine.parse_bullet_list(&lines_bullet).unwrap();
        match list_bullet {
            JiraContent::BulletList { content } => assert_eq!(content.len(), 2),
            _ => panic!("Expected BulletList"),
        }

        // Test with -
        let lines_dash = vec!["- Item 1", "- Item 2"];
        let (list_dash, _) = engine.parse_bullet_list(&lines_dash).unwrap();
        match list_dash {
            JiraContent::BulletList { content } => assert_eq!(content.len(), 2),
            _ => panic!("Expected BulletList"),
        }
    }

    #[test]
    fn test_markdown_to_adf_complete() {
        let engine = TemplateEngine::new();
        let markdown = r#"h2. Test Heading

This is a **bold** paragraph.

* List item 1
* List item 2

```bash
echo "test"
```

Final paragraph with `inline code`.
"#;

        let content = engine.markdown_to_adf(markdown).unwrap();

        // Should have: Heading, Paragraph, BulletList, CodeBlock, Paragraph
        assert!(content.len() >= 5);

        // First should be heading
        assert!(matches!(content[0], JiraContent::Heading { .. }));

        // Second should be paragraph
        assert!(matches!(content[1], JiraContent::Paragraph { .. }));

        // Third should be bullet list
        assert!(matches!(content[2], JiraContent::BulletList { .. }));

        // Fourth should be code block
        assert!(matches!(content[3], JiraContent::CodeBlock { .. }));

        // Fifth should be paragraph
        assert!(matches!(content[4], JiraContent::Paragraph { .. }));
    }

    #[test]
    fn test_render_description_with_variables() {
        let engine = TemplateEngine::with_templates(
            "title".to_string(),
            "h2. Summary\n\n**CVE:** {cve_id}\n**Package:** {package}".to_string(),
        );

        let mut variables = HashMap::new();
        variables.insert("cve_id".to_string(), "CVE-2024-1234".to_string());
        variables.insert("package".to_string(), "log4j-core".to_string());

        let description = engine.render_description(&variables).unwrap();

        assert_eq!(description.doc_type, "doc");
        assert_eq!(description.version, 1);
        assert!(!description.content.is_empty());

        // First content should be heading
        assert!(matches!(
            description.content[0],
            JiraContent::Heading { .. }
        ));
    }

    #[test]
    fn test_render_description_complex_template() {
        let engine = TemplateEngine::new(); // Uses default template

        let mut variables = HashMap::new();
        variables.insert("cve_id".to_string(), "CVE-2024-1234".to_string());
        variables.insert("package".to_string(), "log4j-core".to_string());
        variables.insert("current_version".to_string(), "2.17.0".to_string());
        variables.insert("fix_version".to_string(), "2.20.0".to_string());
        variables.insert("severity".to_string(), "CRITICAL".to_string());
        variables.insert("cvss_score".to_string(), "9.8".to_string());
        variables.insert("priority".to_string(), "P0".to_string());
        variables.insert("reachability_status".to_string(), "⚠ REACHABLE".to_string());
        variables.insert(
            "why_fix".to_string(),
            "Active exploitation detected".to_string(),
        );
        variables.insert(
            "impact_description".to_string(),
            "Remote Code Execution".to_string(),
        );
        variables.insert("epss_score".to_string(), "0.89".to_string());
        variables.insert("kev_status".to_string(), "⚠ ACTIVE".to_string());
        variables.insert("exploit_intel".to_string(), "PoCs available".to_string());
        variables.insert("ml_risk_score".to_string(), "92".to_string());
        variables.insert("ml_risk_level".to_string(), "CRITICAL".to_string());
        variables.insert("ml_risk_breakdown".to_string(), "See table".to_string());
        variables.insert("reachability_confidence".to_string(), "95".to_string());
        variables.insert("call_path_count".to_string(), "3".to_string());
        variables.insert("call_paths".to_string(), "path1\\npath2".to_string());
        variables.insert("affected_files".to_string(), "file1.java".to_string());
        variables.insert("remediation_difficulty".to_string(), "15".to_string());
        variables.insert("remediation_level".to_string(), "Very Easy".to_string());
        variables.insert("remediation_time".to_string(), "45 minutes".to_string());
        variables.insert(
            "remediation_reasons".to_string(),
            "Simple upgrade".to_string(),
        );
        variables.insert(
            "fix_description".to_string(),
            "Upgrade to 2.20.0".to_string(),
        );
        variables.insert("breaking_changes_status".to_string(), "✓ NONE".to_string());
        variables.insert("breaking_changes_details".to_string(), "".to_string());
        variables.insert(
            "fix_command".to_string(),
            "bazbom fix log4j-core".to_string(),
        );
        variables.insert("framework_name".to_string(), "Apache Log4j 2.x".to_string());
        variables.insert("migration_required".to_string(), "None".to_string());
        variables.insert("compatibility_info".to_string(), "Java 8+".to_string());
        variables.insert(
            "test_recommendations".to_string(),
            "Run security tests".to_string(),
        );
        variables.insert("container_image_count".to_string(), "3".to_string());
        variables.insert("container_details".to_string(), "myapp:latest".to_string());
        variables.insert("policy_violations_before".to_string(), "❌ 3".to_string());
        variables.insert("policy_violations_after".to_string(), "✓ 0".to_string());
        variables.insert("compliance_frameworks".to_string(), "PCI-DSS".to_string());
        variables.insert(
            "bazbom_link".to_string(),
            "https://bazbom.local".to_string(),
        );
        variables.insert("cve_link".to_string(), "https://nvd.nist.gov".to_string());
        variables.insert(
            "github_pr_link".to_string(),
            "https://github.com/pr/1".to_string(),
        );
        variables.insert("additional_links".to_string(), "".to_string());
        variables.insert("attachments_list".to_string(), "callgraph.svg".to_string());

        let description = engine.render_description(&variables).unwrap();

        assert_eq!(description.doc_type, "doc");
        assert_eq!(description.version, 1);
        assert!(!description.content.is_empty());

        // Should have multiple headings
        let heading_count = description
            .content
            .iter()
            .filter(|c| matches!(c, JiraContent::Heading { .. }))
            .count();

        assert!(heading_count >= 8); // At least 8 h2 sections in default template
    }
}
