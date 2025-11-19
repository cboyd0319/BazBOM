//! Java entrypoint detection

use super::models::MethodNode;

/// Check if a method is a Java entrypoint
pub fn is_entrypoint(method: &MethodNode) -> bool {
    // main() method
    if is_main_method(method) {
        return true;
    }

    // Servlet methods
    if is_servlet_method(method) {
        return true;
    }

    // Spring controller methods
    if is_spring_controller_method(method) {
        return true;
    }

    // JAX-RS REST endpoints
    if is_jaxrs_endpoint(method) {
        return true;
    }

    // Test methods (optional - may want to exclude these)
    // if is_test_method(method) {
    //     return true;
    // }

    false
}

/// Check if method is main()
fn is_main_method(method: &MethodNode) -> bool {
    method.name == "main"
        && method.is_public
        && method.is_static
        && method.descriptor == "([Ljava/lang/String;)V"
}

/// Check if method is a Servlet method (doGet, doPost, etc.)
fn is_servlet_method(method: &MethodNode) -> bool {
    if !method.is_public {
        return false;
    }

    let servlet_methods = [
        "doGet",
        "doPost",
        "doPut",
        "doDelete",
        "doHead",
        "doOptions",
        "doTrace",
        "service",
        "init",
        "destroy",
    ];

    servlet_methods.contains(&method.name.as_str())
        && (method.class_name.contains("Servlet")
            || method.descriptor.contains("HttpServletRequest")
            || method.descriptor.contains("ServletRequest"))
}

/// Check if method is a Spring Controller method
fn is_spring_controller_method(method: &MethodNode) -> bool {
    if !method.is_public {
        return false;
    }

    // This is a simplified check
    // In a full implementation, we would parse annotations from the class file
    method.class_name.contains("Controller")
        || method.class_name.contains("RestController")
        || method.class_name.contains("Endpoint")
}

/// Check if method is a JAX-RS REST endpoint
fn is_jaxrs_endpoint(method: &MethodNode) -> bool {
    if !method.is_public {
        return false;
    }

    // Simplified check for JAX-RS
    // Full implementation would parse @Path, @GET, @POST annotations
    method.class_name.contains("Resource")
        || method.class_name.contains("Service")
        || method.class_name.contains("Endpoint")
}

/// Check if method is a JUnit test method
#[allow(dead_code)]
fn is_test_method(method: &MethodNode) -> bool {
    method.is_public
        && (method.name.starts_with("test")
            || method.name.contains("Test")
            || method.name.contains("Should"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_main_method_detection() {
        let mut method = MethodNode::new(
            "com.example.Main:main([Ljava/lang/String;)V".to_string(),
            "main".to_string(),
            "com.example.Main".to_string(),
            "([Ljava/lang/String;)V".to_string(),
        );
        method.is_public = true;
        method.is_static = true;

        assert!(is_main_method(&method));
        assert!(is_entrypoint(&method));
    }

    #[test]
    fn test_servlet_method_detection() {
        let mut method = MethodNode::new(
            "com.example.MyServlet:doGet".to_string(),
            "doGet".to_string(),
            "com.example.MyServlet".to_string(),
            "(LHttpServletRequest;LHttpServletResponse;)V".to_string(),
        );
        method.is_public = true;

        assert!(is_servlet_method(&method));
        assert!(is_entrypoint(&method));
    }
}
