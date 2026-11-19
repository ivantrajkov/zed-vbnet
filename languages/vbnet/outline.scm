(namespace_block
  name: (namespace_name) @name) @item

(class_block
  modifiers: (modifiers)? @context
  name: (identifier) @name) @item

(module_block
  modifiers: (modifiers)? @context
  name: (identifier) @name) @item

(structure_block
  modifiers: (modifiers)? @context
  name: (identifier) @name) @item

(interface_block
  modifiers: (modifiers)? @context
  name: (identifier) @name) @item

(enum_block
  modifiers: (modifiers)? @context
  name: (identifier) @name) @item

(enum_member
  name: (identifier) @name) @item

(delegate_declaration
  name: (identifier) @name
  parameters: (parameter_list) @context) @item

(method_declaration
  name: (identifier) @name
  parameters: (parameter_list) @context) @item

(constructor_declaration
  parameters: (parameter_list) @name) @item

(property_declaration
  name: (identifier) @name) @item

(event_declaration
  name: (identifier) @name) @item

(field_declaration
  (variable_declarator
    name: (identifier) @name)) @item

; Member-level constants only (not local constants inside methods).
(class_block (const_declaration name: (identifier) @name) @item)
(module_block (const_declaration name: (identifier) @name) @item)
(structure_block (const_declaration name: (identifier) @name) @item)

