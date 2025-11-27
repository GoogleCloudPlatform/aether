// Copyright 2025 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Documentation renderer for multiple output formats
//!
//! Renders parsed documentation into HTML, Markdown, PDF, and JSON formats
//! with customizable themes and layouts.

use crate::docs::OutputFormat;
use crate::docs::{
    DocConfig, Documentation, Example, ModuleDoc, SearchIndex, ThemeConfig, Tutorial,
};
use crate::error::SemanticError;
use std::collections::HashMap;
use std::path::PathBuf;

/// Documentation renderer
#[derive(Debug)]
pub struct DocRenderer {
    /// Template engine for rendering
    template_engine: TemplateEngine,

    /// Asset manager for static files
    asset_manager: AssetManager,

    /// Output format
    output_format: OutputFormat,
}

/// Template engine for rendering documentation
#[derive(Debug)]
pub struct TemplateEngine {
    /// Loaded templates
    templates: HashMap<String, String>,
}

/// Template structure
#[derive(Debug, Clone)]
pub struct Template {
    /// Template name
    pub name: String,

    /// Template content
    pub content: String,

    /// Template variables
    pub variables: Vec<String>,

    /// Included templates
    pub includes: Vec<String>,
}

/// Template helper function
#[derive(Debug)]
pub struct TemplateHelper {
    /// Helper name
    pub name: String,

    /// Helper function
    pub function: fn(&[String]) -> Result<String, String>,
}

/// Asset manager for static files
#[derive(Debug)]
pub struct AssetManager {}

/// Asset information
#[derive(Debug, Clone)]
pub struct AssetInfo {
    /// Asset path
    pub path: PathBuf,

    /// Asset type
    pub asset_type: AssetType,

    /// Asset size
    pub size: usize,

    /// Asset hash
    pub hash: String,

    /// Processing options
    pub processing: AssetProcessing,
}

/// Asset types
#[derive(Debug, Clone)]
pub enum AssetType {
    Stylesheet,
    JavaScript,
    Image,
    Font,
    Icon,
    Other(String),
}

/// Asset processing options
#[derive(Debug, Clone)]
pub struct AssetProcessing {
    /// Minify asset
    pub minify: bool,

    /// Compress asset
    pub compress: bool,

    /// Generate source maps
    pub source_maps: bool,

    /// Cache bust with hash
    pub cache_bust: bool,
}

/// Asset processor
#[derive(Debug)]
pub struct AssetProcessor {
    /// Processor name
    pub name: String,

    /// File patterns to process
    pub patterns: Vec<String>,

    /// Processing function
    pub process: fn(&str) -> Result<String, String>,
}

/// Theme manager
#[derive(Debug)]
pub struct ThemeManager {}

/// Theme definition
#[derive(Debug, Clone)]
pub struct Theme {
    /// Theme name
    pub name: String,

    /// Theme configuration
    pub config: ThemeConfig,

    /// Template overrides
    pub templates: HashMap<String, String>,

    /// Asset overrides
    pub assets: HashMap<String, PathBuf>,

    /// Custom CSS
    pub custom_css: Option<String>,

    /// Custom JavaScript
    pub custom_js: Option<String>,
}

/// Rendering context
#[derive(Debug, Clone)]
pub struct RenderContext {
    /// Current module path
    pub module_path: String,

    /// Base URL
    pub base_url: String,

    /// Template variables
    pub variables: HashMap<String, String>,

    /// Navigation structure
    pub navigation: NavigationTree,

    /// Search index
    pub search_index: Option<SearchIndex>,
}

/// Navigation tree structure
#[derive(Debug, Clone)]
pub struct NavigationTree {
    /// Navigation items
    pub items: Vec<NavigationItem>,

    /// Current active item
    pub active_item: Option<String>,
}

/// Navigation item
#[derive(Debug, Clone)]
pub struct NavigationItem {
    /// Item title
    pub title: String,

    /// Item URL
    pub url: String,

    /// Item type
    pub item_type: NavigationType,

    /// Child items
    pub children: Vec<NavigationItem>,

    /// Whether item is active
    pub active: bool,
}

/// Navigation types
#[derive(Debug, Clone)]
pub enum NavigationType {
    Module,
    Function,
    Type,
    Tutorial,
    Example,
    Reference,
    External,
}

/// HTML renderer
#[derive(Debug)]
pub struct HtmlRenderer<'a> {
    /// Documentation to render
    doc: &'a Documentation,
}

impl<'a> HtmlRenderer<'a> {
    /// Render a template
    fn render_template(
        &self,
        template_name: &str,
        context: &HashMap<String, String>,
    ) -> Result<String, SemanticError> {
        // Simple template rendering - in a real implementation, this would use a template engine
        let template = match template_name {
            "index" => "<html><body><h1>{{project_name}}</h1><p>{{project_description}}</p></body></html>",
            "module" => "<html><body><h1>{{module_name}}</h1><p>{{module_description}}</p></body></html>",
            "tutorial" => "<html><body><h1>{{tutorial_title}}</h1><div>{{tutorial_content}}</div></body></html>",
            "example" => "<html><body><h1>{{example_title}}</h1><code>{{example_code}}</code></body></html>",
            _ => return Err(SemanticError::Internal {
                message: format!("Unknown template: {}", template_name),
            }),
        };

        let mut result = template.to_string();
        for (key, value) in context {
            result = result.replace(&format!("{{{{{}}}}}", key), value);
        }

        Ok(result)
    }
}

/// Markdown renderer
#[derive(Debug)]
pub struct MarkdownRenderer {}

/// Markdown options
#[derive(Debug, Clone)]
pub struct MarkdownOptions {
    /// GitHub flavored markdown
    pub github_flavored: bool,

    /// Include table of contents
    pub include_toc: bool,

    /// Code highlighting
    pub syntax_highlighting: bool,

    /// Math rendering
    pub math_rendering: bool,
}

/// Table of contents generator
#[derive(Debug)]
pub struct TocGenerator {}

/// TOC formats
#[derive(Debug, Clone)]
pub enum TocFormat {
    List,
    Tree,
    Numbered,
}

/// PDF renderer
#[derive(Debug)]
pub struct PdfRenderer {}

/// PDF options
#[derive(Debug, Clone)]
pub struct PdfOptions {
    /// Page size
    pub page_size: String,

    /// Include bookmarks
    pub bookmarks: bool,

    /// Include page numbers
    pub page_numbers: bool,

    /// Include headers/footers
    pub headers_footers: bool,
}

/// Page layout
#[derive(Debug, Clone)]
pub struct PageLayout {
    /// Margins
    pub margins: Margins,

    /// Font settings
    pub fonts: FontSettings,

    /// Color scheme
    pub colors: ColorScheme,
}

/// Page margins
#[derive(Debug, Clone)]
pub struct Margins {
    pub top: f32,
    pub bottom: f32,
    pub left: f32,
    pub right: f32,
}

/// Font settings
#[derive(Debug, Clone)]
pub struct FontSettings {
    /// Base font family
    pub family: String,

    /// Base font size
    pub size: f32,

    /// Line height
    pub line_height: f32,

    /// Code font family
    pub code_family: String,
}

/// Color scheme
#[derive(Debug, Clone)]
pub struct ColorScheme {
    /// Text color
    pub text: String,

    /// Background color
    pub background: String,

    /// Accent color
    pub accent: String,

    /// Code background
    pub code_background: String,
}

impl DocRenderer {
    /// Create a new documentation renderer
    pub fn new(config: &DocConfig) -> Result<Self, SemanticError> {
        let templates = TemplateEngine::new(&config.output_dir)?;
        let assets = AssetManager::new(&config.output_dir)?;
        let themes = ThemeManager::new()?;

        Ok(Self {
            template_engine: templates,
            asset_manager: assets,
            output_format: config.output_format.clone(),
        })
    }

    /// Render documentation to HTML
    pub fn render_html(
        &mut self,
        docs: &Documentation,
        output_dir: &PathBuf,
    ) -> Result<(), SemanticError> {
        let html_dir = output_dir.join("html");
        std::fs::create_dir_all(&html_dir)?;

        let mut renderer = HtmlRenderer { doc: docs };

        // Render main index page
        renderer.render_index(docs, &html_dir)?;

        // Render API documentation
        renderer.render_api_docs(&docs.api, &html_dir)?;

        // Render tutorials
        for tutorial in &docs.tutorials {
            renderer.render_tutorial(tutorial, &html_dir)?;
        }

        // Render examples
        for example in &docs.examples {
            renderer.render_example(example, &html_dir)?;
        }

        // Render reference manual
        renderer.render_reference(&docs.reference, &html_dir)?;

        // Copy assets
        self.asset_manager.copy_to_output(&html_dir)?;

        // Generate search index
        if let Some(ref search_index) = docs.search_index {
            renderer.render_search_index(search_index, &html_dir)?;
        }

        Ok(())
    }

    /// Render documentation to Markdown
    pub fn render_markdown(
        &mut self,
        docs: &Documentation,
        output_dir: &PathBuf,
    ) -> Result<(), SemanticError> {
        let md_dir = output_dir.join("markdown");
        std::fs::create_dir_all(&md_dir)?;

        let renderer = MarkdownRenderer::new()?;

        // Render API documentation
        renderer.render_api_docs(&docs.api, &md_dir)?;

        // Render tutorials
        for tutorial in &docs.tutorials {
            renderer.render_tutorial(tutorial, &md_dir)?;
        }

        // Render examples
        for example in &docs.examples {
            renderer.render_example(example, &md_dir)?;
        }

        // Render reference manual
        renderer.render_reference(&docs.reference, &md_dir)?;

        Ok(())
    }

    /// Render documentation to PDF
    pub fn render_pdf(
        &mut self,
        docs: &Documentation,
        output_dir: &PathBuf,
    ) -> Result<(), SemanticError> {
        let pdf_dir = output_dir.join("pdf");
        std::fs::create_dir_all(&pdf_dir)?;

        let renderer = PdfRenderer::new()?;

        // Generate single PDF with all documentation
        let pdf_path = pdf_dir.join("documentation.pdf");
        renderer.render_complete_docs(docs, &pdf_path)?;

        Ok(())
    }

    /// Render documentation to JSON
    pub fn render_json(
        &mut self,
        docs: &Documentation,
        output_dir: &PathBuf,
    ) -> Result<(), SemanticError> {
        let json_dir = output_dir.join("json");
        std::fs::create_dir_all(&json_dir)?;

        // Serialize documentation to JSON
        let json_content =
            serde_json::to_string_pretty(docs).map_err(|e| SemanticError::Internal {
                message: format!("Failed to serialize documentation to JSON: {}", e),
            })?;

        let json_path = json_dir.join("documentation.json");
        std::fs::write(json_path, json_content)?;

        // Also create separate files for different sections
        self.render_api_json(&docs.api, &json_dir)?;
        self.render_tutorials_json(&docs.tutorials, &json_dir)?;
        self.render_examples_json(&docs.examples, &json_dir)?;

        Ok(())
    }

    // Helper methods for JSON rendering

    fn render_api_json(
        &self,
        api: &crate::docs::ApiDocumentation,
        output_dir: &PathBuf,
    ) -> Result<(), SemanticError> {
        let api_json = serde_json::to_string_pretty(api).map_err(|e| SemanticError::Internal {
            message: format!("Failed to serialize API docs to JSON: {}", e),
        })?;

        std::fs::write(output_dir.join("api.json"), api_json)?;
        Ok(())
    }

    fn render_tutorials_json(
        &self,
        tutorials: &[Tutorial],
        output_dir: &PathBuf,
    ) -> Result<(), SemanticError> {
        let tutorials_json =
            serde_json::to_string_pretty(tutorials).map_err(|e| SemanticError::Internal {
                message: format!("Failed to serialize tutorials to JSON: {}", e),
            })?;

        std::fs::write(output_dir.join("tutorials.json"), tutorials_json)?;
        Ok(())
    }

    fn render_examples_json(
        &self,
        examples: &[Example],
        output_dir: &PathBuf,
    ) -> Result<(), SemanticError> {
        let examples_json =
            serde_json::to_string_pretty(examples).map_err(|e| SemanticError::Internal {
                message: format!("Failed to serialize examples to JSON: {}", e),
            })?;

        std::fs::write(output_dir.join("examples.json"), examples_json)?;
        Ok(())
    }
}

impl TemplateEngine {
    fn new(base_dir: &PathBuf) -> Result<Self, SemanticError> {
        let mut templates = HashMap::new();
        let search_paths = vec![base_dir.join("templates"), PathBuf::from("templates")];

        // Load default templates
        templates.insert("index".to_string(), Self::create_index_template().content);
        templates.insert("module".to_string(), Self::create_module_template().content);
        templates.insert(
            "function".to_string(),
            Self::create_function_template().content,
        );
        templates.insert("type".to_string(), Self::create_type_template().content);
        templates.insert(
            "tutorial".to_string(),
            Self::create_tutorial_template().content,
        );
        templates.insert(
            "example".to_string(),
            Self::create_example_template().content,
        );

        let helpers = Self::create_default_helpers();

        Ok(Self { templates })
    }

    fn create_index_template() -> Template {
        Template {
            name: "index".to_string(),
            content: r#"
<!DOCTYPE html>
<html>
<head>
    <title>{{project_name}} Documentation</title>
    <link rel="stylesheet" href="assets/style.css">
</head>
<body>
    <header>
        <h1>{{project_name}}</h1>
        <p>{{project_description}}</p>
    </header>
    <nav>
        {{#navigation}}
        <ul>
        {{#items}}
            <li><a href="{{url}}">{{title}}</a></li>
        {{/items}}
        </ul>
        {{/navigation}}
    </nav>
    <main>
        {{content}}
    </main>
</body>
</html>
            "#
            .to_string(),
            variables: vec![
                "project_name".to_string(),
                "project_description".to_string(),
                "content".to_string(),
            ],
            includes: vec![],
        }
    }

    fn create_module_template() -> Template {
        Template {
            name: "module".to_string(),
            content: r#"
<div class="module">
    <h2>{{module_name}}</h2>
    <p>{{module_description}}</p>
    
    {{#functions}}
    <div class="function">
        <h3>{{name}}</h3>
        <code>{{signature}}</code>
        <p>{{description}}</p>
        {{#examples}}
        <pre><code>{{code}}</code></pre>
        {{/examples}}
    </div>
    {{/functions}}
    
    {{#types}}
    <div class="type">
        <h3>{{name}}</h3>
        <p>{{description}}</p>
    </div>
    {{/types}}
</div>
            "#
            .to_string(),
            variables: vec!["module_name".to_string(), "module_description".to_string()],
            includes: vec![],
        }
    }

    fn create_function_template() -> Template {
        Template {
            name: "function".to_string(),
            content: r#"
<div class="function-doc">
    <h1>{{name}}</h1>
    <div class="signature">
        <code>{{signature}}</code>
    </div>
    <div class="description">
        {{description}}
    </div>
    {{#parameters}}
    <div class="parameters">
        <h3>Parameters</h3>
        {{#param_list}}
        <div class="parameter">
            <code>{{name}}</code>: {{type}} - {{description}}
        </div>
        {{/param_list}}
    </div>
    {{/parameters}}
    {{#examples}}
    <div class="examples">
        <h3>Examples</h3>
        {{#example_list}}
        <pre><code class="language-{{language}}">{{code}}</code></pre>
        {{/example_list}}
    </div>
    {{/examples}}
</div>
            "#
            .to_string(),
            variables: vec![
                "name".to_string(),
                "signature".to_string(),
                "description".to_string(),
            ],
            includes: vec![],
        }
    }

    fn create_type_template() -> Template {
        Template {
            name: "type".to_string(),
            content: r#"
<div class="type-doc">
    <h1>{{name}}</h1>
    <div class="description">
        {{description}}
    </div>
    {{#fields}}
    <div class="fields">
        <h3>Fields</h3>
        {{#field_list}}
        <div class="field">
            <code>{{name}}</code>: {{type}} - {{description}}
        </div>
        {{/field_list}}
    </div>
    {{/fields}}
    {{#methods}}
    <div class="methods">
        <h3>Methods</h3>
        {{#method_list}}
        <div class="method">
            <a href="{{url}}">{{name}}</a>
        </div>
        {{/method_list}}
    </div>
    {{/methods}}
</div>
            "#
            .to_string(),
            variables: vec!["name".to_string(), "description".to_string()],
            includes: vec![],
        }
    }

    fn create_tutorial_template() -> Template {
        Template {
            name: "tutorial".to_string(),
            content: r#"
<div class="tutorial">
    <h1>{{title}}</h1>
    <div class="meta">
        <span class="difficulty">{{difficulty}}</span>
        <span class="time">{{estimated_time}}</span>
    </div>
    <div class="description">
        {{description}}
    </div>
    <div class="content">
        {{content}}
    </div>
    {{#sections}}
    <div class="section">
        <h2>{{title}}</h2>
        <div class="section-content">
            {{content}}
        </div>
        {{#examples}}
        <div class="example">
            <pre><code class="language-{{language}}">{{code}}</code></pre>
        </div>
        {{/examples}}
    </div>
    {{/sections}}
</div>
            "#
            .to_string(),
            variables: vec![
                "title".to_string(),
                "description".to_string(),
                "content".to_string(),
            ],
            includes: vec![],
        }
    }

    fn create_example_template() -> Template {
        Template {
            name: "example".to_string(),
            content: r#"
<div class="example">
    <h1>{{name}}</h1>
    <div class="description">
        {{description}}
    </div>
    <div class="category">
        Category: {{category}}
    </div>
    <div class="source-code">
        <h3>Source Code</h3>
        <pre><code>{{source_code}}</code></pre>
    </div>
    {{#expected_output}}
    <div class="output">
        <h3>Expected Output</h3>
        <pre>{{expected_output}}</pre>
    </div>
    {{/expected_output}}
    {{#build_instructions}}
    <div class="build">
        <h3>Build Instructions</h3>
        <pre>{{build_instructions}}</pre>
    </div>
    {{/build_instructions}}
</div>
            "#
            .to_string(),
            variables: vec![
                "name".to_string(),
                "description".to_string(),
                "source_code".to_string(),
            ],
            includes: vec![],
        }
    }

    fn create_default_helpers() -> HashMap<String, TemplateHelper> {
        let mut helpers = HashMap::new();

        helpers.insert(
            "escape".to_string(),
            TemplateHelper {
                name: "escape".to_string(),
                function: |args| {
                    if let Some(text) = args.first() {
                        Ok(text
                            .replace('&', "&amp;")
                            .replace('<', "&lt;")
                            .replace('>', "&gt;")
                            .replace('"', "&quot;")
                            .replace('\'', "&#x27;"))
                    } else {
                        Err("escape helper requires one argument".to_string())
                    }
                },
            },
        );

        helpers.insert(
            "uppercase".to_string(),
            TemplateHelper {
                name: "uppercase".to_string(),
                function: |args| {
                    if let Some(text) = args.first() {
                        Ok(text.to_uppercase())
                    } else {
                        Err("uppercase helper requires one argument".to_string())
                    }
                },
            },
        );

        helpers.insert(
            "lowercase".to_string(),
            TemplateHelper {
                name: "lowercase".to_string(),
                function: |args| {
                    if let Some(text) = args.first() {
                        Ok(text.to_lowercase())
                    } else {
                        Err("lowercase helper requires one argument".to_string())
                    }
                },
            },
        );

        helpers
    }

    fn render_template(
        &self,
        template_name: &str,
        context: &HashMap<String, String>,
    ) -> Result<String, SemanticError> {
        let template =
            self.templates
                .get(template_name)
                .ok_or_else(|| SemanticError::Internal {
                    message: format!("Template '{}' not found", template_name),
                })?;

        // Simple template rendering - in real implementation would use proper template engine
        let mut rendered = template.clone();

        for (key, value) in context {
            let placeholder = format!("{{{{{}}}}}", key);
            rendered = rendered.replace(&placeholder, value);
        }

        Ok(rendered)
    }
}

impl AssetManager {
    fn new(base_dir: &PathBuf) -> Result<Self, SemanticError> {
        let assets_dir = base_dir.join("assets");
        std::fs::create_dir_all(&assets_dir)?;

        let mut assets = HashMap::new();
        let processors: Vec<&dyn Fn(&str) -> String> = vec![];

        // Add default CSS
        assets.insert(
            "style.css".to_string(),
            AssetInfo {
                path: assets_dir.join("style.css"),
                asset_type: AssetType::Stylesheet,
                size: 0,
                hash: String::new(),
                processing: AssetProcessing {
                    minify: true,
                    compress: true,
                    source_maps: false,
                    cache_bust: true,
                },
            },
        );

        Ok(Self {})
    }

    fn copy_to_output(&self, output_dir: &PathBuf) -> Result<(), SemanticError> {
        let assets_output = output_dir.join("assets");
        std::fs::create_dir_all(&assets_output)?;

        // Create default CSS file
        let default_css = self.create_default_css();
        std::fs::write(assets_output.join("style.css"), default_css)?;

        Ok(())
    }

    fn create_default_css(&self) -> String {
        r#"
/* Default documentation styles */
body {
    font-family: system-ui, -apple-system, sans-serif;
    line-height: 1.6;
    color: #333;
    max-width: 1200px;
    margin: 0 auto;
    padding: 20px;
}

header {
    border-bottom: 2px solid #eee;
    margin-bottom: 30px;
    padding-bottom: 20px;
}

nav ul {
    list-style: none;
    padding: 0;
    display: flex;
    gap: 20px;
}

nav a {
    text-decoration: none;
    color: #007acc;
    font-weight: 500;
}

nav a:hover {
    text-decoration: underline;
}

.module, .function-doc, .type-doc, .tutorial, .example {
    margin-bottom: 40px;
    padding: 20px;
    border: 1px solid #eee;
    border-radius: 8px;
}

.signature, .source-code {
    background: #f8f9fa;
    padding: 15px;
    border-radius: 5px;
    font-family: 'Fira Code', monospace;
    overflow-x: auto;
}

.description {
    margin: 15px 0;
}

.parameters, .fields, .methods, .examples {
    margin-top: 20px;
}

.parameter, .field, .method {
    padding: 10px;
    margin: 5px 0;
    background: #f8f9fa;
    border-left: 3px solid #007acc;
}

.meta {
    display: flex;
    gap: 15px;
    margin-bottom: 15px;
}

.difficulty, .time {
    background: #e9ecef;
    padding: 4px 8px;
    border-radius: 4px;
    font-size: 0.9em;
}

code {
    background: #f1f3f4;
    padding: 2px 4px;
    border-radius: 3px;
    font-family: 'Fira Code', monospace;
}

pre {
    background: #f8f9fa;
    padding: 15px;
    border-radius: 5px;
    overflow-x: auto;
}

h1, h2, h3 {
    color: #2c3e50;
}

a {
    color: #007acc;
}

a:hover {
    color: #005999;
}
        "#
        .to_string()
    }
}

impl ThemeManager {
    fn new() -> Result<Self, SemanticError> {
        let mut themes = HashMap::new();

        // Add default theme
        themes.insert(
            "default".to_string(),
            Theme {
                name: "default".to_string(),
                config: ThemeConfig::default(),
                templates: HashMap::new(),
                assets: HashMap::new(),
                custom_css: None,
                custom_js: None,
            },
        );

        Ok(Self {})
    }
}

impl<'a> HtmlRenderer<'a> {
    fn render_index(
        &mut self,
        docs: &Documentation,
        output_dir: &PathBuf,
    ) -> Result<(), SemanticError> {
        let mut context = HashMap::new();
        context.insert("project_name".to_string(), docs.metadata.name.clone());
        context.insert(
            "project_description".to_string(),
            docs.metadata
                .description
                .as_deref()
                .unwrap_or("")
                .to_string(),
        );

        let rendered = self.render_template("index", &context)?;
        std::fs::write(output_dir.join("index.html"), rendered)?;

        Ok(())
    }

    fn render_api_docs(
        &mut self,
        api: &crate::docs::ApiDocumentation,
        output_dir: &PathBuf,
    ) -> Result<(), SemanticError> {
        let api_dir = output_dir.join("api");
        std::fs::create_dir_all(&api_dir)?;

        // Render each module
        for module in &api.modules {
            self.render_module_doc(module, &api_dir)?;
        }

        Ok(())
    }

    fn render_module_doc(
        &mut self,
        module: &ModuleDoc,
        output_dir: &PathBuf,
    ) -> Result<(), SemanticError> {
        let mut context = HashMap::new();
        context.insert("module_name".to_string(), module.name.clone());
        context.insert(
            "module_description".to_string(),
            module.description.as_deref().unwrap_or("").to_string(),
        );

        let rendered = self.render_template("module", &context)?;
        let filename = format!("{}.html", module.name);
        std::fs::write(output_dir.join(filename), rendered)?;

        Ok(())
    }

    fn render_tutorial(
        &mut self,
        tutorial: &Tutorial,
        output_dir: &PathBuf,
    ) -> Result<(), SemanticError> {
        let tutorials_dir = output_dir.join("tutorials");
        std::fs::create_dir_all(&tutorials_dir)?;

        let mut context = HashMap::new();
        context.insert("title".to_string(), tutorial.title.clone());
        context.insert("description".to_string(), tutorial.description.clone());
        context.insert("content".to_string(), tutorial.content.clone());

        let rendered = self.render_template("tutorial", &context)?;
        let filename = format!("{}.html", tutorial.title.to_lowercase().replace(' ', "_"));
        std::fs::write(tutorials_dir.join(filename), rendered)?;

        Ok(())
    }

    fn render_example(
        &mut self,
        example: &Example,
        output_dir: &PathBuf,
    ) -> Result<(), SemanticError> {
        let examples_dir = output_dir.join("examples");
        std::fs::create_dir_all(&examples_dir)?;

        let mut context = HashMap::new();
        context.insert("name".to_string(), example.name.clone());
        context.insert("description".to_string(), example.description.clone());
        context.insert("source_code".to_string(), example.source_code.clone());

        let rendered = self.render_template("example", &context)?;
        let filename = format!("{}.html", example.name.to_lowercase().replace(' ', "_"));
        std::fs::write(examples_dir.join(filename), rendered)?;

        Ok(())
    }

    fn render_reference(
        &mut self,
        reference: &crate::docs::ReferenceManual,
        output_dir: &PathBuf,
    ) -> Result<(), SemanticError> {
        let reference_dir = output_dir.join("reference");
        std::fs::create_dir_all(&reference_dir)?;

        // Render reference sections
        for section in &reference.sections {
            self.render_reference_section(section, &reference_dir)?;
        }

        Ok(())
    }

    fn render_reference_section(
        &mut self,
        section: &crate::docs::ManualSection,
        output_dir: &PathBuf,
    ) -> Result<(), SemanticError> {
        let content = format!("<h1>{}</h1>\n<div>{}</div>", section.title, section.content);
        let filename = format!("{}.html", section.title.to_lowercase().replace(' ', "_"));
        std::fs::write(output_dir.join(filename), content)?;

        Ok(())
    }

    fn render_search_index(
        &mut self,
        search_index: &SearchIndex,
        output_dir: &PathBuf,
    ) -> Result<(), SemanticError> {
        let search_data =
            serde_json::to_string(search_index).map_err(|e| SemanticError::Internal {
                message: format!("Failed to serialize search index: {}", e),
            })?;

        std::fs::write(output_dir.join("search-index.json"), search_data)?;

        Ok(())
    }
}

impl MarkdownRenderer {
    fn new() -> Result<Self, SemanticError> {
        let options = MarkdownOptions {
            github_flavored: true,
            include_toc: true,
            syntax_highlighting: true,
            math_rendering: false,
        };

        let toc_generator = TocGenerator {};

        Ok(Self {})
    }

    fn render_api_docs(
        &self,
        api: &crate::docs::ApiDocumentation,
        output_dir: &PathBuf,
    ) -> Result<(), SemanticError> {
        let api_dir = output_dir.join("api");
        std::fs::create_dir_all(&api_dir)?;

        // Render API index
        let mut content = String::new();
        content.push_str("# API Documentation\n\n");

        for module in &api.modules {
            content.push_str(&format!(
                "## [{}]({})\n\n",
                module.name,
                format!("{}.md", module.name)
            ));
            if let Some(ref description) = module.description {
                content.push_str(&format!("{}\n\n", description));
            }
        }

        std::fs::write(api_dir.join("index.md"), content)?;

        // Render individual modules
        for module in &api.modules {
            self.render_module_md(module, &api_dir)?;
        }

        Ok(())
    }

    fn render_module_md(
        &self,
        module: &ModuleDoc,
        output_dir: &PathBuf,
    ) -> Result<(), SemanticError> {
        let mut content = String::new();
        content.push_str(&format!("# {}\n\n", module.name));

        if let Some(ref description) = module.description {
            content.push_str(&format!("{}\n\n", description));
        }

        content.push_str(&module.docs);
        content.push_str("\n\n");

        for item in &module.items {
            content.push_str(&format!("## {}\n\n", item.name));
            if let Some(ref description) = item.description {
                content.push_str(&format!("{}\n\n", description));
            }
        }

        let filename = format!("{}.md", module.name);
        std::fs::write(output_dir.join(filename), content)?;

        Ok(())
    }

    fn render_tutorial(
        &self,
        tutorial: &Tutorial,
        output_dir: &PathBuf,
    ) -> Result<(), SemanticError> {
        let tutorials_dir = output_dir.join("tutorials");
        std::fs::create_dir_all(&tutorials_dir)?;

        let mut content = String::new();
        content.push_str(&format!("# {}\n\n", tutorial.title));
        content.push_str(&format!("{}\n\n", tutorial.description));

        // Add metadata
        content.push_str(&format!("**Difficulty:** {:?}\n\n", tutorial.difficulty));
        if let Some(ref time) = tutorial.estimated_time {
            content.push_str(&format!("**Estimated Time:** {}\n\n", time));
        }

        content.push_str(&tutorial.content);
        content.push_str("\n\n");

        for section in &tutorial.sections {
            content.push_str(&format!("## {}\n\n", section.title));
            content.push_str(&format!("{}\n\n", section.content));

            for example in &section.examples {
                content.push_str("```aetherscript\n");
                content.push_str(&example.code);
                content.push_str("\n```\n\n");
            }
        }

        let filename = format!("{}.md", tutorial.title.to_lowercase().replace(' ', "_"));
        std::fs::write(tutorials_dir.join(filename), content)?;

        Ok(())
    }

    fn render_example(&self, example: &Example, output_dir: &PathBuf) -> Result<(), SemanticError> {
        let examples_dir = output_dir.join("examples");
        std::fs::create_dir_all(&examples_dir)?;

        let mut content = String::new();
        content.push_str(&format!("# {}\n\n", example.name));
        content.push_str(&format!("{}\n\n", example.description));
        content.push_str(&format!("**Category:** {}\n\n", example.category));

        content.push_str("## Source Code\n\n");
        content.push_str("```aetherscript\n");
        content.push_str(&example.source_code);
        content.push_str("\n```\n\n");

        if let Some(ref output) = example.expected_output {
            content.push_str("## Expected Output\n\n");
            content.push_str("```\n");
            content.push_str(output);
            content.push_str("\n```\n\n");
        }

        let filename = format!("{}.md", example.name.to_lowercase().replace(' ', "_"));
        std::fs::write(examples_dir.join(filename), content)?;

        Ok(())
    }

    fn render_reference(
        &self,
        reference: &crate::docs::ReferenceManual,
        output_dir: &PathBuf,
    ) -> Result<(), SemanticError> {
        let reference_dir = output_dir.join("reference");
        std::fs::create_dir_all(&reference_dir)?;

        let mut content = String::new();
        content.push_str("# Reference Manual\n\n");

        for section in &reference.sections {
            content.push_str(&format!("## {}\n\n", section.title));
            content.push_str(&format!("{}\n\n", section.content));
        }

        std::fs::write(reference_dir.join("index.md"), content)?;

        Ok(())
    }
}

impl PdfRenderer {
    fn new() -> Result<Self, SemanticError> {
        let options = PdfOptions {
            page_size: "A4".to_string(),
            bookmarks: true,
            page_numbers: true,
            headers_footers: true,
        };

        let layout = PageLayout {
            margins: Margins {
                top: 72.0,
                bottom: 72.0,
                left: 72.0,
                right: 72.0,
            },
            fonts: FontSettings {
                family: "DejaVu Sans".to_string(),
                size: 11.0,
                line_height: 1.4,
                code_family: "DejaVu Sans Mono".to_string(),
            },
            colors: ColorScheme {
                text: "#000000".to_string(),
                background: "#ffffff".to_string(),
                accent: "#007acc".to_string(),
                code_background: "#f8f9fa".to_string(),
            },
        };

        Ok(Self {})
    }

    fn render_complete_docs(
        &self,
        docs: &Documentation,
        output_path: &PathBuf,
    ) -> Result<(), SemanticError> {
        // Simplified PDF generation - in real implementation would use PDF library
        let mut content = String::new();

        content.push_str(&format!("# {} Documentation\n\n", docs.metadata.name));

        if let Some(ref description) = docs.metadata.description {
            content.push_str(&format!("{}\n\n", description));
        }

        // Add API documentation
        content.push_str("# API Documentation\n\n");
        for module in &docs.api.modules {
            content.push_str(&format!("## Module: {}\n\n", module.name));
            content.push_str(&module.docs);
            content.push_str("\n\n");
        }

        // Add tutorials
        content.push_str("# Tutorials\n\n");
        for tutorial in &docs.tutorials {
            content.push_str(&format!("## {}\n\n", tutorial.title));
            content.push_str(&tutorial.content);
            content.push_str("\n\n");
        }

        // Save as text file (placeholder for PDF generation)
        std::fs::write(output_path, content)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_engine_creation() {
        let base_dir = PathBuf::from("test");
        let engine = TemplateEngine::new(&base_dir).unwrap();

        assert!(engine.templates.contains_key("index"));
        assert!(engine.templates.contains_key("module"));
    }

    #[test]
    fn test_asset_manager() {
        let base_dir = PathBuf::from("test");
        let manager = AssetManager::new(&base_dir).unwrap();

        // AssetManager created successfully
    }

    #[test]
    fn test_html_renderer_creation() {
        // HtmlRenderer requires a Documentation reference
        // This test would need a proper Documentation struct to work
    }

    #[test]
    fn test_markdown_renderer() {
        let renderer = MarkdownRenderer::new().unwrap();
        // Renderer created successfully
    }

    #[test]
    fn test_pdf_renderer() {
        let renderer = PdfRenderer::new().unwrap();
        // Renderer created successfully
    }
}
