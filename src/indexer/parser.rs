use anyhow::Result;

/// Represents a code symbol (function, class, method)
#[derive(Debug, Clone)]
pub struct CodeSymbol {
    pub name: String,
    pub kind: SymbolKind,
    pub start_line: usize,
    pub end_line: usize,
    pub doc_comment: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SymbolKind {
    Function,
    Class,
    Method,
    Struct,
    Interface,
    Enum,
}

/// AST wrapper — currently uses fallback line-based parsing
/// In production, this would use tree-sitter for full AST parsing
pub struct AstParser;

/// Convenience alias
pub type CodeParser = AstParser;

impl AstParser {
    pub fn new() -> Result<Self> {
        Ok(Self)
    }

    /// Parse source code into an AST representation
    pub fn parse(&self, source: &str, _lang: &str) -> Result<Vec<CodeSymbol>> {
        self.extract_symbols(source)
    }

    /// Extract symbols using heuristics (line-based)
    /// Replace with tree-sitter queries for production
    pub fn extract_symbols(&self, source: &str) -> Result<Vec<CodeSymbol>> {
        let mut symbols = Vec::new();
        let lines: Vec<&str> = source.lines().collect();

        // Pattern matcher: returns Some(name) if line matches pattern, None otherwise.
        // Handles simple prefix patterns as well as special cases (Go types, arrow functions, typedef).
        let match_pattern = |trimmed: &str, pattern: &str| -> Option<String> {
            if *pattern == *"type " && trimmed.starts_with("type ") && trimmed.contains(" struct") {
                // Go: "type X struct"
                trimmed.strip_prefix("type ")?
                    .split(|c: char| c == ' ' || c == '{')
                    .next()
                    .map(|s| s.to_string())
                    .filter(|s| !s.is_empty())
            } else if *pattern == *"type.interface" && trimmed.starts_with("type ") && trimmed.contains(" interface") {
                // Go: "type X interface"
                trimmed.strip_prefix("type ")?
                    .split(|c: char| c == ' ' || c == '{')
                    .next()
                    .map(|s| s.to_string())
                    .filter(|s| !s.is_empty())
            } else if *pattern == *"const.*=>" && trimmed.starts_with("const ") && trimmed.contains("=>") {
                // JS/TS arrow functions: "const X = (...) =>"
                let after_const = trimmed.strip_prefix("const ")?;
                after_const.split(|c: char| c == ' ' || c == '=' || c == '(')
                    .next()
                    .map(|s| s.to_string())
                    .filter(|s| !s.is_empty())
            } else if *pattern == *"typedef struct" && trimmed.starts_with("typedef struct") {
                // C/C++/C#: "typedef struct X"
                let rest = trimmed.strip_prefix("typedef struct")?.trim();
                rest.split(|c: char| c == ' ' || c == '{')
                    .next()
                    .map(|s| s.to_string())
                    .filter(|s| !s.is_empty())
            } else if trimmed.starts_with(pattern) {
                // Default: simple prefix matching
                let name = trimmed.strip_prefix(pattern)?
                    .split(|c: char| c == '(' || c == '{' || c == ' ' || c == '<' || c == ':')
                    .next()
                    .unwrap_or("")
                    .to_string();
                if name.is_empty() { None } else { Some(name) }
            } else {
                None
            }
        };

        // Simple pattern matching for common language constructs
        // Extends across Rust, Python, JS/TS, Go, Java, C++, Ruby, Swift, Kotlin
        let patterns: Vec<(SymbolKind, &str)> = vec![
            // Rust
            (SymbolKind::Function, "fn "),
            (SymbolKind::Struct, "struct "),
            (SymbolKind::Enum, "enum "),
            (SymbolKind::Method, "impl "),
            (SymbolKind::Interface, "trait "),
            // Python
            (SymbolKind::Function, "def "),
            (SymbolKind::Class, "class "),
            // JavaScript / TypeScript
            (SymbolKind::Function, "function "),
            (SymbolKind::Function, "const.*=>"),   // arrow functions
            (SymbolKind::Class, "class "),
            (SymbolKind::Interface, "interface "),
            (SymbolKind::Enum, "enum "),
            // Go
            (SymbolKind::Function, "func "),
            (SymbolKind::Struct, "type "),           // will check for " struct"
            (SymbolKind::Interface, "type.interface"), // will check for " interface"
            // Java / C++ / C#
            (SymbolKind::Class, "public class "),
            (SymbolKind::Class, "private class "),
            (SymbolKind::Class, "protected class "),
            (SymbolKind::Interface, "public interface "),
            (SymbolKind::Enum, "public enum "),
            (SymbolKind::Function, "public static "),
            (SymbolKind::Function, "private static "),
            (SymbolKind::Function, "public "),         // Java methods
            (SymbolKind::Function, "private "),        // Java methods
            (SymbolKind::Function, "protected "),      // Java methods
            (SymbolKind::Struct, "typedef struct"),
            // Ruby
            (SymbolKind::Function, "def "),
            (SymbolKind::Class, "class "),
            (SymbolKind::Method, "def self."),
            // Swift
            (SymbolKind::Function, "func "),
            (SymbolKind::Class, "class "),
            (SymbolKind::Struct, "struct "),
            (SymbolKind::Enum, "enum "),
            (SymbolKind::Interface, "protocol "),
            // Kotlin
            (SymbolKind::Function, "fun "),
            (SymbolKind::Class, "class "),
            (SymbolKind::Interface, "interface "),
            (SymbolKind::Enum, "enum class "),
            (SymbolKind::Struct, "data class "),
        ];

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            for (kind, pattern) in &patterns {
                if let Some(name) = match_pattern(trimmed, pattern) {

                    if !name.is_empty() {
                        // Find end of block by counting braces
                        let mut brace_count = 0;
                        let mut end_line = i;
                        for j in i..lines.len() {
                            for c in lines[j].chars() {
                                match c {
                                    '{' => brace_count += 1,
                                    '}' => {
                                        brace_count -= 1;
                                        if brace_count == 0 {
                                            end_line = j;
                                            break;
                                        }
                                    }
                                    _ => {}
                                }
                            }
                            if brace_count == 0 && j > i {
                                end_line = j;
                                break;
                            }
                        }

                        // Look for doc comment above
                        let doc_comment = if i > 0 {
                            let prev = lines[i - 1].trim();
                            if prev.starts_with("///") || prev.starts_with("# ") || prev.starts_with("// ") {
                                Some(prev.trim_start_matches(|c: char| c == '/' || c == '#' || c == ' ').to_string())
                            } else {
                                None
                            }
                        } else {
                            None
                        };

                        symbols.push(CodeSymbol {
                            name,
                            kind: kind.clone(),
                            start_line: i,
                            end_line,
                            doc_comment,
                        });
                    }
                }
            }
        }

        Ok(symbols)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_rust_functions() {
        let parser = CodeParser::new().unwrap();
        let source = r#"
fn hello() {
    println!("Hello");
}

fn add(a: i32, b: i32) -> i32 {
    a + b
}

struct Point {
    x: f64,
    y: f64,
}

impl Point {
    fn new(x: f64, y: f64) -> Self {
        Point { x, y }
    }
}
"#;
        let symbols = parser.extract_symbols(source).unwrap();
        assert!(!symbols.is_empty(), "Should find at least one symbol");

        let names: Vec<&str> = symbols.iter().map(|s| s.name.as_str()).collect();
        assert!(names.contains(&"hello"), "Should find hello function");
        assert!(names.contains(&"add"), "Should find add function");
        assert!(names.contains(&"Point"), "Should find Point struct");

        // Verify doc_comment and kind
        let hello = symbols.iter().find(|s| s.name == "hello").unwrap();
        assert_eq!(hello.kind, SymbolKind::Function);
    }

    #[test]
    fn test_parse_python_functions() {
        let parser = CodeParser::new().unwrap();
        let source = r#"
def greet(name):
    print(f"Hello {name}")

class Calculator:
    def add(self, a, b):
        return a + b

    def subtract(self, a, b):
        return a - b
"#;
        let symbols = parser.extract_symbols(source).unwrap();
        let names: Vec<&str> = symbols.iter().map(|s| s.name.as_str()).collect();
        assert!(names.contains(&"greet"));
        assert!(names.contains(&"Calculator"));
    }

    #[test]
    fn test_parse_javascript() {
        let parser = CodeParser::new().unwrap();
        let source = r#"
function calculateTotal(items) {
    return items.reduce((sum, item) => sum + item.price, 0);
}

class ShoppingCart {
    constructor() {
        this.items = [];
    }
}
"#;
        let symbols = parser.extract_symbols(source).unwrap();
        let names: Vec<&str> = symbols.iter().map(|s| s.name.as_str()).collect();
        assert!(names.contains(&"calculateTotal"));
        assert!(names.contains(&"ShoppingCart"));
    }

    #[test]
    fn test_parse_go() {
        let parser = CodeParser::new().unwrap();
        let source = r#"
func main() {
    fmt.Println("Hello")
}

type User struct {
    Name string
    Age  int
}

func (u *User) Greet() string {
    return "Hello " + u.Name
}
"#;
        let symbols = parser.extract_symbols(source).unwrap();
        let names: Vec<&str> = symbols.iter().map(|s| s.name.as_str()).collect();
        assert!(names.contains(&"main"));
        assert!(names.contains(&"User"));
    }

    #[test]
    fn test_parse_java() {
        let parser = CodeParser::new().unwrap();
        let source = r#"
public class HelloWorld {
    private String name;

    public void sayHello() {
        System.out.println("Hello!");
    }

    public static void main(String[] args) {
        new HelloWorld().sayHello();
    }
}
"#;
        let symbols = parser.extract_symbols(source).unwrap();
        let names: Vec<&str> = symbols.iter().map(|s| s.name.as_str()).collect();
        assert!(names.contains(&"HelloWorld"));
    }

    #[test]
    fn test_empty_source() {
        let parser = CodeParser::new().unwrap();
        let symbols = parser.extract_symbols("").unwrap();
        assert!(symbols.is_empty());
    }

    #[test]
    fn test_no_symbols() {
        let parser = CodeParser::new().unwrap();
        let symbols = parser.extract_symbols("just some text\nwithout any\nfunctions").unwrap();
        assert!(symbols.is_empty());
    }

    #[test]
    fn test_doc_comment() {
        let parser = CodeParser::new().unwrap();
        let source = r#"
/// This is a greeting function
fn hello() {
    println!("Hello");
}

// This is a regular comment
fn world() {
    println!("World");
}
"#;
        let symbols = parser.extract_symbols(source).unwrap();
        let hello = symbols.iter().find(|s| s.name == "hello").unwrap();
        assert_eq!(hello.doc_comment.as_deref(), Some("This is a greeting function"));
    }

    #[test]
    fn test_brace_counting() {
        let parser = CodeParser::new().unwrap();
        let source = r#"
fn complex() {
    if true {
        loop {
            break;
        }
    }
}
"#;
        let symbols = parser.extract_symbols(source).unwrap();
        let complex = symbols.iter().find(|s| s.name == "complex").unwrap();
        // The function body spans from line 1 to line 7 (0-indexed)
        assert!(complex.end_line >= complex.start_line);
        // end_line should be the last line of the function
        assert_eq!(complex.end_line, 7);
    }

    #[test]
    fn test_parse_rust_enum() {
        let parser = CodeParser::new().unwrap();
        let source = r#"
enum Color {
    Red,
    Green,
    Blue,
}
"#;
        let symbols = parser.extract_symbols(source).unwrap();
        let names: Vec<&str> = symbols.iter().map(|s| s.name.as_str()).collect();
        assert!(names.contains(&"Color"));
    }

    #[test]
    fn test_parse_kotlin() {
        let parser = CodeParser::new().unwrap();
        let source = r#"
fun main() {
    println("Hello Kotlin")
}

data class User(val name: String, val age: Int)

enum class Status { ACTIVE, INACTIVE }
"#;
        let symbols = parser.extract_symbols(source).unwrap();
        let names: Vec<&str> = symbols.iter().map(|s| s.name.as_str()).collect();
        assert!(names.contains(&"main"), "Should find kotlin main function");
    }
}
