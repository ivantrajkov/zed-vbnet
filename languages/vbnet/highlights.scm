; VB.NET syntax highlighting.
;
; The upstream grammar (tree-sitter-vb-dotnet) matches keywords with hidden
; tokens, so words like `Class`, `Sub`, `If`, `End` are not nodes and cannot be
; captured here. Keyword colouring comes from the vb-ls semantic tokens
; (enable "semantic_tokens": "combined" for VB.NET). Statements made only of
; keywords plus child expressions are captured whole as @keyword below; their
; children override with more specific captures.
;
; Later patterns take precedence over earlier ones.

(identifier) @variable

; Keyword-only statements and clauses.
[
  (option_statements)
  (imports_statement)
  (exit_statement)
  (continue_statement)
  (return_statement)
  (throw_statement)
  (goto_statement)
  (modifier)
] @keyword

((literal) @constant.builtin
  (#match? @constant.builtin "^[Nn][Oo][Tt][Hh][Ii][Nn][Gg]$"))

; Types
(primitive_type) @type.builtin
(type (namespace_name (identifier) @type .))
(generic_type (namespace_name (identifier) @type .))
(array_type (namespace_name (identifier) @type .))
(type_parameter name: (identifier) @type)

(class_block name: (identifier) @type)
(module_block name: (identifier) @type)
(structure_block name: (identifier) @type)
(interface_block name: (identifier) @type)
(enum_block name: (identifier) @type)
(delegate_declaration name: (identifier) @type)

(enum_member name: (identifier) @constant)
(const_declaration name: (identifier) @constant)

; Namespaces
(namespace_block name: (namespace_name (identifier) @namespace))
(imports_statement namespace: (namespace_name (identifier) @namespace))

; Members
(method_declaration name: (identifier) @function)
(property_declaration name: (identifier) @property)
(event_declaration name: (identifier) @property)
(member_access member: (identifier) @property)
(member_initializer member: (identifier) @property)
(object_initializer (identifier) @property)

(invocation target: (identifier) @function)
(invocation target: (member_access member: (identifier) @function.method))

; Parameters and labels
(parameter name: (identifier) @variable.parameter)
(lambda_parameter name: (identifier) @variable.parameter)
(argument name: (identifier) @variable.parameter)
(label_statement label: (identifier) @label)
(goto_statement label: (identifier) @label)

; Attributes
(attribute name: (identifier) @attribute)
(attribute name: (namespace_name (identifier) @attribute))

; Literals
(boolean_literal) @boolean
(integer_literal) @number
(floating_point_literal) @number
(string_literal) @string
(character_literal) @string
(date_literal) @string.special
(interpolation
  "{" @punctuation.special
  "}" @punctuation.special) @embedded

; Comments and directives
(comment) @comment
((comment) @comment.doc
  (#match? @comment.doc "^'''"))
(preprocessor_directive) @preproc

; Operators and punctuation
(relational_operator) @operator
[
  "="
  "+"
  "-"
  "*"
  "/"
  "\\"
  "^"
  "<<"
  ">>"
  "<>"
  "<="
  ">="
  "+="
  "-="
  "*="
  "/="
  "\\="
  "^="
  "&="
  "<<="
  ">>="
] @operator

(binary_expression ["<" ">"] @operator)
(attribute_block ["<" ">"] @punctuation.bracket)

["(" ")" "{" "}"] @punctuation.bracket
["," "." ":"] @punctuation.delimiter
