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

//! Example generation and management for AetherScript
//!
//! Generates comprehensive code examples covering language features,
//! standard library usage, and common programming patterns.

#![allow(dead_code)]

use crate::docs::{DocConfig, Example};
use crate::error::SemanticError;
use std::collections::HashMap;

/// Example manager for generating and organizing code examples
#[derive(Debug)]
pub struct ExampleManager {
    /// Loaded examples
    examples: Vec<Example>,
}

/// Example category
#[derive(Debug, Clone)]
pub struct ExampleCategory {
    /// Category name
    pub name: String,

    /// Category description
    pub description: String,

    /// Category examples
    pub examples: Vec<String>,

    /// Category difficulty level
    pub difficulty: ExampleDifficulty,

    /// Prerequisites
    pub prerequisites: Vec<String>,
}

/// Example difficulty levels
#[derive(Debug, Clone)]
pub enum ExampleDifficulty {
    Beginner,
    Intermediate,
    Advanced,
    Expert,
}

/// Example templates for generating code
#[derive(Debug)]
pub struct ExampleTemplates {
    /// Basic syntax examples
    pub syntax: Vec<ExampleTemplate>,

    /// Data structure examples
    pub data_structures: Vec<ExampleTemplate>,

    /// Algorithm examples
    pub algorithms: Vec<ExampleTemplate>,

    /// Standard library examples
    pub stdlib: Vec<ExampleTemplate>,

    /// Advanced feature examples
    pub advanced: Vec<ExampleTemplate>,
}

/// Example template
#[derive(Debug, Clone)]
pub struct ExampleTemplate {
    /// Template name
    pub name: String,

    /// Template description
    pub description: String,

    /// Code template
    pub code_template: String,

    /// Expected output template
    pub output_template: Option<String>,

    /// Template variables
    pub variables: Vec<TemplateVariable>,

    /// Build instructions
    pub build_instructions: Option<String>,

    /// Tags
    pub tags: Vec<String>,
}

/// Template variable
#[derive(Debug, Clone)]
pub struct TemplateVariable {
    /// Variable name
    pub name: String,

    /// Variable type
    pub var_type: VariableType,

    /// Default value
    pub default_value: String,

    /// Description
    pub description: String,
}

/// Variable types for templates
#[derive(Debug, Clone)]
pub enum VariableType {
    String,
    Number,
    Boolean,
    List(Box<VariableType>),
    Custom(String),
}

impl ExampleManager {
    /// Create a new example manager
    pub fn new(_config: &DocConfig) -> Result<Self, SemanticError> {
        Ok(Self {
            examples: Vec::new(),
        })
    }

    /// Generate all examples
    pub fn generate_examples(&mut self) -> Result<Vec<Example>, SemanticError> {
        self.examples.clear();

        // Generate basic syntax examples
        self.generate_syntax_examples()?;

        // Generate data structure examples
        self.generate_data_structure_examples()?;

        // Generate algorithm examples
        self.generate_algorithm_examples()?;

        // Generate standard library examples
        self.generate_stdlib_examples()?;

        // Generate advanced feature examples
        self.generate_advanced_examples()?;

        // Generate real-world application examples
        self.generate_application_examples()?;

        Ok(self.examples.clone())
    }

    /// Generate basic syntax examples
    pub fn generate_syntax_examples(&mut self) -> Result<(), SemanticError> {
        // Hello World example
        self.examples.push(Example {
            name: "Hello World".to_string(),
            description: "A simple hello world program in AetherScript".to_string(),
            category: "Basic Syntax".to_string(),
            source_code: r#";;; Hello World in AetherScript
(println "Hello, World!")"#
                .to_string(),
            expected_output: Some("Hello, World!".to_string()),
            dependencies: vec![],
            build_instructions: Some("aether run hello_world.aether".to_string()),
            tags: vec!["basic".to_string(), "output".to_string()],
        });

        // Variables and data types
        self.examples.push(Example {
            name: "Variables and Types".to_string(),
            description: "Demonstrating variable declarations and basic data types".to_string(),
            category: "Basic Syntax".to_string(),
            source_code: r#";;; Variables and data types
(def name "Alice")        ; String
(def age 30)              ; Integer
(def height 5.6)          ; Float
(def is-student true)     ; Boolean
(def hobbies ["reading" "coding" "gaming"])  ; List

(println "Name:" name)
(println "Age:" age)
(println "Height:" height)
(println "Student?" is-student)
(println "Hobbies:" hobbies)"#
                .to_string(),
            expected_output: Some(
                r#"Name: Alice
Age: 30
Height: 5.6
Student? true
Hobbies: ["reading" "coding" "gaming"]"#
                    .to_string(),
            ),
            dependencies: vec![],
            build_instructions: Some("aether run variables.aether".to_string()),
            tags: vec![
                "variables".to_string(),
                "types".to_string(),
                "basic".to_string(),
            ],
        });

        // Functions
        self.examples.push(Example {
            name: "Functions".to_string(),
            description: "Defining and calling functions with parameters and return values"
                .to_string(),
            category: "Basic Syntax".to_string(),
            source_code: r#";;; Function definitions and calls
(defn greet [name]
  "Greets a person by name"
  (str "Hello, " name "!"))

(defn add [x y]
  "Adds two numbers"
  (+ x y))

(defn factorial [n]
  "Calculates factorial recursively"
  (if (<= n 1)
    1
    (* n (factorial (- n 1)))))

(println (greet "World"))
(println "5 + 3 =" (add 5 3))
(println "5! =" (factorial 5))"#
                .to_string(),
            expected_output: Some(
                r#"Hello, World!
5 + 3 = 8
5! = 120"#
                    .to_string(),
            ),
            dependencies: vec![],
            build_instructions: Some("aether run functions.aether".to_string()),
            tags: vec![
                "functions".to_string(),
                "recursion".to_string(),
                "basic".to_string(),
            ],
        });

        // Control flow
        self.examples.push(Example {
            name: "Control Flow".to_string(),
            description: "Conditional expressions and loops".to_string(),
            category: "Basic Syntax".to_string(),
            source_code: r#";;; Control flow examples
(defn classify-number [n]
  "Classifies a number as positive, negative, or zero"
  (cond
    (> n 0) "positive"
    (< n 0) "negative"
    :else "zero"))

(defn count-down [n]
  "Counts down from n to 1"
  (loop [i n]
    (when (> i 0)
      (println i)
      (recur (- i 1)))))

(defn sum-range [start end]
  "Sums numbers in a range"
  (loop [i start sum 0]
    (if (<= i end)
      (recur (+ i 1) (+ sum i))
      sum)))

(println (classify-number 5))
(println (classify-number -3))
(println (classify-number 0))

(count-down 5)
(println "Sum 1-10:" (sum-range 1 10))"#
                .to_string(),
            expected_output: Some(
                r#"positive
negative
zero
5
4
3
2
1
Sum 1-10: 55"#
                    .to_string(),
            ),
            dependencies: vec![],
            build_instructions: Some("aether run control_flow.aether".to_string()),
            tags: vec![
                "control-flow".to_string(),
                "loops".to_string(),
                "conditionals".to_string(),
            ],
        });

        Ok(())
    }

    /// Generate data structure examples
    pub fn generate_data_structure_examples(&mut self) -> Result<(), SemanticError> {
        // Lists and vectors
        self.examples.push(Example {
            name: "Working with Lists".to_string(),
            description: "List operations and manipulations".to_string(),
            category: "Data Structures".to_string(),
            source_code: r#";;; List operations
(def numbers [1 2 3 4 5])
(def fruits ["apple" "banana" "cherry"])

;;; Basic operations
(println "Numbers:" numbers)
(println "First number:" (first numbers))
(println "Last number:" (last numbers))
(println "Rest:" (rest numbers))

;;; List manipulation
(def more-numbers (cons 0 numbers))
(def doubled (map #(* 2 %) numbers))
(def evens (filter even? numbers))
(def sum (reduce + numbers))

(println "With 0 prepended:" more-numbers)
(println "Doubled:" doubled)
(println "Even numbers:" evens)
(println "Sum:" sum)

;;; List comprehension
(def squares (map #(* % %) (range 1 6)))
(println "Squares 1-5:" squares)"#
                .to_string(),
            expected_output: Some(
                r#"Numbers: [1 2 3 4 5]
First number: 1
Last number: 5
Rest: [2 3 4 5]
With 0 prepended: [0 1 2 3 4 5]
Doubled: [2 4 6 8 10]
Even numbers: [2 4]
Sum: 15
Squares 1-5: [1 4 9 16 25]"#
                    .to_string(),
            ),
            dependencies: vec![],
            build_instructions: Some("aether run lists.aether".to_string()),
            tags: vec![
                "lists".to_string(),
                "collections".to_string(),
                "functional".to_string(),
            ],
        });

        // Maps and dictionaries
        self.examples.push(Example {
            name: "Working with Maps".to_string(),
            description: "Map operations and key-value data structures".to_string(),
            category: "Data Structures".to_string(),
            source_code: r#";;; Map operations
(def person {:name "Alice"
             :age 30
             :city "New York"
             :hobbies ["reading" "coding"]})

;;; Accessing values
(println "Name:" (:name person))
(println "Age:" (get person :age))
(println "Country:" (get person :country "Unknown"))

;;; Updating maps
(def updated-person (assoc person :age 31 :country "USA"))
(def no-city (dissoc person :city))

(println "Updated person:" updated-person)
(println "Without city:" no-city)

;;; Map operations
(def keys (keys person))
(def values (vals person))
(println "Keys:" keys)
(println "Values:" values)

;;; Nested access
(def company {:name "TechCorp"
              :employees [{:name "Alice" :role "Developer"}
                         {:name "Bob" :role "Designer"}]})

(println "First employee:" (-> company :employees first :name))"#.to_string(),
            expected_output: Some(r#"Name: Alice
Age: 30
Country: Unknown
Updated person: {:name "Alice", :age 31, :city "New York", :hobbies ["reading" "coding"], :country "USA"}
Without city: {:name "Alice", :age 30, :hobbies ["reading" "coding"]}
Keys: [:name :age :city :hobbies]
Values: ["Alice" 30 "New York" ["reading" "coding"]]
First employee: Alice"#.to_string()),
            dependencies: vec![],
            build_instructions: Some("aether run maps.aether".to_string()),
            tags: vec!["maps".to_string(), "dictionaries".to_string(), "key-value".to_string()],
        });

        // Sets
        self.examples.push(Example {
            name: "Working with Sets".to_string(),
            description: "Set operations and unique collections".to_string(),
            category: "Data Structures".to_string(),
            source_code: r#";;; Set operations
(def set1 #{1 2 3 4 5})
(def set2 #{4 5 6 7 8})
(def list-with-dupes [1 2 2 3 3 3 4])

;;; Basic set operations
(println "Set1:" set1)
(println "Set2:" set2)
(println "Contains 3?" (contains? set1 3))

;;; Set from list (removes duplicates)
(def unique-set (set list-with-dupes))
(println "Unique from list:" unique-set)

;;; Set operations
(def union-set (union set1 set2))
(def intersection-set (intersection set1 set2))
(def difference-set (difference set1 set2))

(println "Union:" union-set)
(println "Intersection:" intersection-set)
(println "Difference:" difference-set)

;;; Adding/removing elements
(def extended-set (conj set1 6 7))
(def reduced-set (disj set1 1 2))

(println "Extended:" extended-set)
(println "Reduced:" reduced-set)"#
                .to_string(),
            expected_output: Some(
                r#"Set1: #{1 2 3 4 5}
Set2: #{4 5 6 7 8}
Contains 3? true
Unique from list: #{1 2 3 4}
Union: #{1 2 3 4 5 6 7 8}
Intersection: #{4 5}
Difference: #{1 2 3}
Extended: #{1 2 3 4 5 6 7}
Reduced: #{3 4 5}"#
                    .to_string(),
            ),
            dependencies: vec![],
            build_instructions: Some("aether run sets.aether".to_string()),
            tags: vec![
                "sets".to_string(),
                "collections".to_string(),
                "unique".to_string(),
            ],
        });

        Ok(())
    }

    /// Generate algorithm examples
    pub fn generate_algorithm_examples(&mut self) -> Result<(), SemanticError> {
        // Sorting algorithms
        self.examples.push(Example {
            name: "Sorting Algorithms".to_string(),
            description: "Implementation of various sorting algorithms".to_string(),
            category: "Algorithms".to_string(),
            source_code: r#";;; Sorting algorithm implementations

(defn bubble-sort [arr]
  "Bubble sort implementation"
  (loop [data (vec arr)
         n (count data)]
    (if (<= n 1)
      data
      (let [swapped (loop [i 0
                          current data
                          made-swap false]
                     (if (>= i (- n 1))
                       [current made-swap]
                       (if (> (nth current i) (nth current (+ i 1)))
                         (recur (+ i 1)
                               (assoc current
                                     i (nth current (+ i 1))
                                     (+ i 1) (nth current i))
                               true)
                         (recur (+ i 1) current made-swap))))]
        (if (second swapped)
          (recur (first swapped) (- n 1))
          (first swapped))))))

(defn quick-sort [arr]
  "Quick sort implementation"
  (if (<= (count arr) 1)
    arr
    (let [pivot (first arr)
          rest-arr (rest arr)
          smaller (filter #(<= % pivot) rest-arr)
          larger (filter #(> % pivot) rest-arr)]
      (concat (quick-sort smaller) [pivot] (quick-sort larger)))))

(defn merge-sort [arr]
  "Merge sort implementation"
  (if (<= (count arr) 1)
    arr
    (let [mid (/ (count arr) 2)
          left (take mid arr)
          right (drop mid arr)]
      (merge-sorted (merge-sort left) (merge-sort right)))))

(defn merge-sorted [left right]
  "Merge two sorted arrays"
  (loop [l left r right result []]
    (cond
      (empty? l) (concat result r)
      (empty? r) (concat result l)
      (<= (first l) (first r)) (recur (rest l) r (conj result (first l)))
      :else (recur l (rest r) (conj result (first r))))))

;;; Test the algorithms
(def test-data [64 34 25 12 22 11 90])

(println "Original:" test-data)
(println "Bubble sort:" (bubble-sort test-data))
(println "Quick sort:" (quick-sort test-data))
(println "Merge sort:" (merge-sort test-data))"#
                .to_string(),
            expected_output: Some(
                r#"Original: [64 34 25 12 22 11 90]
Bubble sort: [11 12 22 25 34 64 90]
Quick sort: [11 12 22 25 34 64 90]
Merge sort: [11 12 22 25 34 64 90]"#
                    .to_string(),
            ),
            dependencies: vec![],
            build_instructions: Some("aether run sorting.aether".to_string()),
            tags: vec![
                "algorithms".to_string(),
                "sorting".to_string(),
                "performance".to_string(),
            ],
        });

        // Search algorithms
        self.examples.push(Example {
            name: "Search Algorithms".to_string(),
            description: "Linear and binary search implementations".to_string(),
            category: "Algorithms".to_string(),
            source_code: r#";;; Search algorithm implementations

(defn linear-search [arr target]
  "Linear search - O(n) time complexity"
  (loop [i 0]
    (cond
      (>= i (count arr)) -1
      (= (nth arr i) target) i
      :else (recur (+ i 1)))))

(defn binary-search [arr target]
  "Binary search - O(log n) time complexity, requires sorted array"
  (loop [left 0
         right (- (count arr) 1)]
    (if (> left right)
      -1
      (let [mid (/ (+ left right) 2)
            mid-val (nth arr mid)]
        (cond
          (= mid-val target) mid
          (< mid-val target) (recur (+ mid 1) right)
          :else (recur left (- mid 1)))))))

(defn interpolation-search [arr target]
  "Interpolation search - better than binary for uniformly distributed data"
  (loop [low 0
         high (- (count arr) 1)]
    (if (or (> low high)
            (< target (nth arr low))
            (> target (nth arr high)))
      -1
      (if (= (nth arr low) (nth arr high))
        (if (= (nth arr low) target) low -1)
        (let [pos (+ low
                    (/ (* (- target (nth arr low))
                          (- high low))
                       (- (nth arr high) (nth arr low))))
              pos-val (nth arr pos)]
          (cond
            (= pos-val target) pos
            (< pos-val target) (recur (+ pos 1) high)
            :else (recur low (- pos 1))))))))

;;; Test the search algorithms
(def sorted-data [11 12 22 25 34 64 90])
(def target 25)

(println "Array:" sorted-data)
(println "Target:" target)
(println "Linear search result:" (linear-search sorted-data target))
(println "Binary search result:" (binary-search sorted-data target))
(println "Interpolation search result:" (interpolation-search sorted-data target))

;;; Search for non-existent element
(def missing-target 50)
(println "\nSearching for missing element:" missing-target)
(println "Linear search result:" (linear-search sorted-data missing-target))
(println "Binary search result:" (binary-search sorted-data missing-target))"#
                .to_string(),
            expected_output: Some(
                r#"Array: [11 12 22 25 34 64 90]
Target: 25
Linear search result: 3
Binary search result: 3
Interpolation search result: 3

Searching for missing element: 50
Linear search result: -1
Binary search result: -1"#
                    .to_string(),
            ),
            dependencies: vec![],
            build_instructions: Some("aether run searching.aether".to_string()),
            tags: vec![
                "algorithms".to_string(),
                "searching".to_string(),
                "binary-search".to_string(),
            ],
        });

        Ok(())
    }

    /// Generate standard library examples
    pub fn generate_stdlib_examples(&mut self) -> Result<(), SemanticError> {
        // String manipulation
        self.examples.push(Example {
            name: "String Manipulation".to_string(),
            description: "Working with strings using standard library functions".to_string(),
            category: "Standard Library".to_string(),
            source_code: r#";;; String manipulation examples
(require '[aether.string :as str])

(def text "Hello, AetherScript World!")
(def words ["apple" "banana" "cherry"])

;;; Basic string operations
(println "Original:" text)
(println "Length:" (count text))
(println "Uppercase:" (str/upper-case text))
(println "Lowercase:" (str/lower-case text))

;;; String predicates
(println "Starts with 'Hello'?" (str/starts-with? text "Hello"))
(println "Ends with 'World!'?" (str/ends-with? text "World!"))
(println "Contains 'Script'?" (str/includes? text "Script"))

;;; String splitting and joining
(def sentence "The quick brown fox")
(def split-words (str/split sentence #" "))
(println "Split words:" split-words)
(println "Joined with '-':" (str/join "-" split-words))

;;; String replacement
(def replaced (str/replace text "World" "Universe"))
(println "Replaced:" replaced)

;;; String trimming
(def padded-text "   trim me   ")
(println "Trimmed:" (str/trim padded-text))

;;; String formatting
(def name "Alice")
(def age 30)
(def formatted (str/format "Name: %s, Age: %d" name age))
(println "Formatted:" formatted)

;;; Regular expressions
(def email "user@example.com")
(def email-pattern #"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}")
(println "Valid email?" (re-matches email-pattern email))"#
                .to_string(),
            expected_output: Some(
                r#"Original: Hello, AetherScript World!
Length: 24
Uppercase: HELLO, AETHERSCRIPT WORLD!
Lowercase: hello, aetherscript world!
Starts with 'Hello'? true
Ends with 'World!'? true
Contains 'Script'? true
Split words: ["The" "quick" "brown" "fox"]
Joined with '-': The-quick-brown-fox
Replaced: Hello, AetherScript Universe!
Trimmed: trim me
Formatted: Name: Alice, Age: 30
Valid email? true"#
                    .to_string(),
            ),
            dependencies: vec!["aether.string".to_string()],
            build_instructions: Some("aether run string_ops.aether".to_string()),
            tags: vec![
                "strings".to_string(),
                "stdlib".to_string(),
                "text-processing".to_string(),
            ],
        });

        // File I/O
        self.examples.push(Example {
            name: "File Input/Output".to_string(),
            description: "Reading from and writing to files".to_string(),
            category: "Standard Library".to_string(),
            source_code: r#";;; File I/O examples
(require '[aether.io :as io])

;;; Writing to a file
(def data ["Line 1" "Line 2" "Line 3"])
(def filename "example.txt")

(with-open [writer (io/writer filename)]
  (doseq [line data]
    (.write-line writer line)))

(println "Data written to" filename)

;;; Reading from a file
(with-open [reader (io/reader filename)]
  (println "File contents:")
  (loop [line (.read-line reader)]
    (when line
      (println "  " line)
      (recur (.read-line reader)))))

;;; Reading entire file at once
(def file-content (slurp filename))
(println "\nEntire file content:")
(println file-content)

;;; Writing entire content at once
(def new-content "This is new content\nWith multiple lines\n")
(spit "new-file.txt" new-content)
(println "New content written to new-file.txt")

;;; Working with file paths
(def path "some/nested/directory/file.txt")
(println "Directory:" (io/parent-path path))
(println "Filename:" (io/filename path))
(println "Extension:" (io/file-extension path))

;;; File system operations
(when (io/exists? filename)
  (println "File size:" (io/file-size filename) "bytes")
  (println "Last modified:" (io/last-modified filename)))

;;; CSV file example
(def csv-data [["Name" "Age" "City"]
               ["Alice" "30" "New York"]
               ["Bob" "25" "San Francisco"]])

(with-open [writer (io/writer "data.csv")]
  (doseq [row csv-data]
    (.write-line writer (str/join "," row))))

(println "CSV data written to data.csv")"#
                .to_string(),
            expected_output: Some(
                r#"Data written to example.txt
File contents:
   Line 1
   Line 2
   Line 3

Entire file content:
Line 1
Line 2
Line 3

New content written to new-file.txt
Directory: some/nested/directory
Filename: file.txt
Extension: txt
File size: 21 bytes
Last modified: 2024-01-15T10:30:00Z
CSV data written to data.csv"#
                    .to_string(),
            ),
            dependencies: vec!["aether.io".to_string(), "aether.string".to_string()],
            build_instructions: Some("aether run file_io.aether".to_string()),
            tags: vec!["io".to_string(), "files".to_string(), "stdlib".to_string()],
        });

        Ok(())
    }

    /// Generate advanced feature examples
    pub fn generate_advanced_examples(&mut self) -> Result<(), SemanticError> {
        // Concurrency example
        self.examples.push(Example {
            name: "Concurrency with Channels".to_string(),
            description: "Using channels for concurrent programming".to_string(),
            category: "Advanced Features".to_string(),
            source_code: r#";;; Concurrency examples with channels
(require '[aether.async :as async])

;;; Producer-consumer pattern
(defn producer [ch items]
  "Produces items and sends them to channel"
  (async/go
    (doseq [item items]
      (println "Producing:" item)
      (async/>! ch item)
      (async/<! (async/timeout 100))) ; Small delay
    (async/close! ch)))

(defn consumer [ch name]
  "Consumes items from channel"
  (async/go-loop []
    (when-let [item (async/<! ch)]
      (println name "consumed:" item)
      (async/<! (async/timeout 50)) ; Processing time
      (recur))))

;;; Create channel and start producer/consumer
(def items-ch (async/chan 10))
(def items [1 2 3 4 5])

(producer items-ch items)
(consumer items-ch "Consumer-1")
(consumer items-ch "Consumer-2")

;;; Wait for completion
(async/<!! (async/timeout 1000))

;;; Pipeline processing
(defn transform-pipeline [input-ch]
  "Creates a processing pipeline"
  (let [step1-ch (async/chan 10)
        step2-ch (async/chan 10)
        output-ch (async/chan 10)]
    
    ; Step 1: Double the values
    (async/go-loop []
      (when-let [val (async/<! input-ch)]
        (async/>! step1-ch (* val 2))
        (recur)))
    
    ; Step 2: Add 10
    (async/go-loop []
      (when-let [val (async/<! step1-ch)]
        (async/>! step2-ch (+ val 10))
        (recur)))
    
    ; Step 3: Convert to string
    (async/go-loop []
      (when-let [val (async/<! step2-ch)]
        (async/>! output-ch (str "Result: " val))
        (recur)))
    
    output-ch))

;;; Test pipeline
(def pipeline-input (async/chan 10))
(def pipeline-output (transform-pipeline pipeline-input))

(async/go
  (doseq [n (range 5)]
    (async/>! pipeline-input n))
  (async/close! pipeline-input))

(async/go-loop []
  (when-let [result (async/<! pipeline-output)]
    (println result)
    (recur)))

(async/<!! (async/timeout 500))"#
                .to_string(),
            expected_output: Some(
                r#"Producing: 1
Consumer-1 consumed: 1
Producing: 2
Consumer-2 consumed: 2
Producing: 3
Consumer-1 consumed: 3
Producing: 4
Consumer-2 consumed: 4
Producing: 5
Consumer-1 consumed: 5
Result: 10
Result: 12
Result: 14
Result: 16
Result: 18"#
                    .to_string(),
            ),
            dependencies: vec!["aether.async".to_string()],
            build_instructions: Some("aether run concurrency.aether".to_string()),
            tags: vec![
                "concurrency".to_string(),
                "channels".to_string(),
                "async".to_string(),
                "advanced".to_string(),
            ],
        });

        // Metaprogramming example
        self.examples.push(Example {
            name: "Metaprogramming with Macros".to_string(),
            description: "Creating and using macros for code generation".to_string(),
            category: "Advanced Features".to_string(),
            source_code: r#";;; Metaprogramming examples

;;; Simple macro
(defmacro when-not [condition & body]
  "Execute body when condition is false"
  `(if (not ~condition)
     (do ~@body)))

;;; Usage of when-not macro
(def x 5)
(when-not (> x 10)
  (println "x is not greater than 10")
  (println "x is" x))

;;; Logging macro
(defmacro log [level message & args]
  "Conditional logging macro"
  `(when (>= *log-level* ~(case level
                            :debug 0
                            :info 1
                            :warn 2
                            :error 3))
     (println "[" ~(name level) "]" ~message ~@args)))

;;; Set log level
(def ^:dynamic *log-level* 1) ; Info level

(log :debug "This won't be printed")
(log :info "This will be printed" "with extra info")
(log :error "This is an error message")

;;; Code generation macro
(defmacro defstruct [name & fields]
  "Creates a struct-like data type with constructor and accessors"
  (let [constructor-name (symbol (str "make-" name))
        field-names (vec fields)
        accessor-fns (map (fn [field]
                           `(defn ~(symbol (str name "-" field)) [obj#]
                              (get obj# ~(keyword field))))
                         fields)]
    `(do
       (defn ~constructor-name [~@fields]
         ~(zipmap (map keyword fields) fields))
       ~@accessor-fns)))

;;; Use the struct macro
(defstruct person name age email)

;;; Create and use person struct
(def alice (make-person "Alice" 30 "alice@example.com"))
(println "Person:" alice)
(println "Name:" (person-name alice))
(println "Age:" (person-age alice))
(println "Email:" (person-email alice))

;;; Benchmarking macro
(defmacro time-it [expr]
  "Times the execution of an expression"
  `(let [start# (System/nanoTime)
         result# ~expr
         end# (System/nanoTime)]
     (println "Execution time:" (/ (- end# start#) 1000000.0) "ms")
     result#))

;;; Test timing
(time-it (reduce + (range 1000000)))

;;; Pattern matching macro (simplified)
(defmacro match [expr & clauses]
  "Simple pattern matching"
  (let [sym (gensym)]
    `(let [~sym ~expr]
       (cond
         ~@(mapcat (fn [[pattern result]]
                    [(if (= pattern :else)
                       true
                       `(= ~sym ~pattern))
                     result])
                  (partition 2 clauses))))))

;;; Test pattern matching
(defn describe-number [n]
  (match n
    0 "zero"
    1 "one"
    2 "two"
    :else "many"))

(println (describe-number 0))
(println (describe-number 1))
(println (describe-number 5))"#
                .to_string(),
            expected_output: Some(
                r#"x is not greater than 10
x is 5
[ info ] This will be printed with extra info
[ error ] This is an error message
Person: {:name "Alice", :age 30, :email "alice@example.com"}
Name: Alice
Age: 30
Email: alice@example.com
Execution time: 2.5 ms
499999500000
zero
one
many"#
                    .to_string(),
            ),
            dependencies: vec![],
            build_instructions: Some("aether run metaprogramming.aether".to_string()),
            tags: vec![
                "macros".to_string(),
                "metaprogramming".to_string(),
                "code-generation".to_string(),
                "advanced".to_string(),
            ],
        });

        Ok(())
    }

    /// Generate real-world application examples
    pub fn generate_application_examples(&mut self) -> Result<(), SemanticError> {
        // Web server example
        self.examples.push(Example {
            name: "Simple Web Server".to_string(),
            description: "Basic HTTP server implementation".to_string(),
            category: "Applications".to_string(),
            source_code: r#";;; Simple web server example
(require '[aether.http :as http])
(require '[aether.json :as json])
(require '[aether.string :as str])

;;; Route handlers
(defn home-handler [request]
  "Home page handler"
  {:status 200
   :headers {"Content-Type" "text/html"}
   :body "<h1>Welcome to AetherScript Web Server</h1>
          <p>Available endpoints:</p>
          <ul>
            <li><a href='/api/hello'>/api/hello</a> - JSON API</li>
            <li><a href='/api/time'>/api/time</a> - Current time</li>
            <li><a href='/api/users'>/api/users</a> - User list</li>
          </ul>"})

(defn hello-handler [request]
  "JSON API hello endpoint"
  (let [name (get-in request [:params :name] "World")]
    {:status 200
     :headers {"Content-Type" "application/json"}
     :body (json/encode {:message (str "Hello, " name "!")
                        :timestamp (System/currentTimeMillis)})}))

(defn time-handler [request]
  "Current time endpoint"
  {:status 200
   :headers {"Content-Type" "application/json"}
   :body (json/encode {:time (java.time.LocalDateTime/now)
                      :timezone "UTC"})})

(defn users-handler [request]
  "Users list endpoint"
  (let [users [{:id 1 :name "Alice" :email "alice@example.com"}
               {:id 2 :name "Bob" :email "bob@example.com"}
               {:id 3 :name "Charlie" :email "charlie@example.com"}]]
    {:status 200
     :headers {"Content-Type" "application/json"}
     :body (json/encode {:users users :count (count users)})}))

(defn not-found-handler [request]
  "404 handler"
  {:status 404
   :headers {"Content-Type" "text/html"}
   :body "<h1>404 - Page Not Found</h1>"})

;;; Route table
(def routes
  [["GET" "/" home-handler]
   ["GET" "/api/hello" hello-handler]
   ["GET" "/api/time" time-handler]
   ["GET" "/api/users" users-handler]])

;;; Simple router
(defn route-request [request]
  "Routes requests to appropriate handlers"
  (let [method (:request-method request)
        uri (:uri request)]
    (if-let [handler (some (fn [[route-method route-path route-handler]]
                            (when (and (= method route-method)
                                      (= uri route-path))
                              route-handler))
                          routes)]
      (handler request)
      (not-found-handler request))))

;;; Middleware
(defn logging-middleware [handler]
  "Logs requests"
  (fn [request]
    (let [start (System/currentTimeMillis)
          response (handler request)
          duration (- (System/currentTimeMillis) start)]
      (println (str (:request-method request) " " (:uri request) 
                   " - " (:status response) " (" duration "ms)"))
      response)))

(defn cors-middleware [handler]
  "Adds CORS headers"
  (fn [request]
    (let [response (handler request)]
      (update response :headers merge
              {"Access-Control-Allow-Origin" "*"
               "Access-Control-Allow-Methods" "GET, POST, PUT, DELETE"
               "Access-Control-Allow-Headers" "Content-Type"}))))

;;; Create server with middleware
(def app (-> route-request
            logging-middleware
            cors-middleware))

;;; Start server
(defn start-server []
  "Starts the web server"
  (println "Starting server on port 8080...")
  (http/start-server app {:port 8080})
  (println "Server started! Visit http://localhost:8080"))

;;; Example usage
(comment
  ; Start the server
  (start-server)
  
  ; Test endpoints
  (http/get "http://localhost:8080/")
  (http/get "http://localhost:8080/api/hello?name=Alice")
  (http/get "http://localhost:8080/api/time")
  (http/get "http://localhost:8080/api/users"))"#
                .to_string(),
            expected_output: Some(
                r#"Starting server on port 8080...
Server started! Visit http://localhost:8080
GET / - 200 (5ms)
GET /api/hello - 200 (2ms)
GET /api/time - 200 (1ms)
GET /api/users - 200 (3ms)"#
                    .to_string(),
            ),
            dependencies: vec![
                "aether.http".to_string(),
                "aether.json".to_string(),
                "aether.string".to_string(),
            ],
            build_instructions: Some("aether run web_server.aether".to_string()),
            tags: vec![
                "web".to_string(),
                "http".to_string(),
                "server".to_string(),
                "api".to_string(),
                "application".to_string(),
            ],
        });

        Ok(())
    }

    // Helper methods

    fn create_example_categories() -> HashMap<String, ExampleCategory> {
        let mut categories = HashMap::new();

        categories.insert(
            "Basic Syntax".to_string(),
            ExampleCategory {
                name: "Basic Syntax".to_string(),
                description: "Fundamental language constructs and syntax".to_string(),
                examples: vec![],
                difficulty: ExampleDifficulty::Beginner,
                prerequisites: vec![],
            },
        );

        categories.insert(
            "Data Structures".to_string(),
            ExampleCategory {
                name: "Data Structures".to_string(),
                description: "Working with collections and data structures".to_string(),
                examples: vec![],
                difficulty: ExampleDifficulty::Intermediate,
                prerequisites: vec!["Basic Syntax".to_string()],
            },
        );

        categories.insert(
            "Algorithms".to_string(),
            ExampleCategory {
                name: "Algorithms".to_string(),
                description: "Common algorithms and problem-solving patterns".to_string(),
                examples: vec![],
                difficulty: ExampleDifficulty::Intermediate,
                prerequisites: vec!["Basic Syntax".to_string(), "Data Structures".to_string()],
            },
        );

        categories.insert(
            "Standard Library".to_string(),
            ExampleCategory {
                name: "Standard Library".to_string(),
                description: "Using standard library functions and modules".to_string(),
                examples: vec![],
                difficulty: ExampleDifficulty::Intermediate,
                prerequisites: vec!["Basic Syntax".to_string()],
            },
        );

        categories.insert(
            "Advanced Features".to_string(),
            ExampleCategory {
                name: "Advanced Features".to_string(),
                description: "Advanced language features and patterns".to_string(),
                examples: vec![],
                difficulty: ExampleDifficulty::Advanced,
                prerequisites: vec!["Basic Syntax".to_string(), "Data Structures".to_string()],
            },
        );

        categories.insert(
            "Applications".to_string(),
            ExampleCategory {
                name: "Applications".to_string(),
                description: "Real-world application examples".to_string(),
                examples: vec![],
                difficulty: ExampleDifficulty::Advanced,
                prerequisites: vec!["Standard Library".to_string()],
            },
        );

        categories
    }

    fn create_example_templates() -> ExampleTemplates {
        ExampleTemplates {
            syntax: vec![],
            data_structures: vec![],
            algorithms: vec![],
            stdlib: vec![],
            advanced: vec![],
        }
    }
}


#[cfg(test)]
mod examples_tests;
