# Types and Members

**Status:** defined core for the initial class and constructor example; partial type system.

See [syntax](../syntax.md), [construction](construction.md), and foundation Sections 4–8 in [SPEC.md](../../SPEC.md).

## 1. Nominal Class Types

A class declaration introduces a distinct nominal type. `Animal` and `Cat` are different types even if a compiler can optimize some of their values into similar machine representations.

For a class `C`, the semantic model records:

- `base(C)`: its optional direct base class.
- `own_fields(C)`: an ordered map of declared fields to their types and visibility.
- Its constructor declarations, instance methods, and implemented interfaces.
- Its inherited contracts and place in the acyclic class hierarchy.

The initial example establishes:

| Property | `Animal` | `Cat` |
| --- | --- | --- |
| Direct base | None. | `Animal`. |
| Own fields | Public `age: u32`. | Public `color: u8`. |
| Inherited field storage | None. | The single `Animal.age` field in its base subobject. |
| Constructor result | A complete `Animal`. | A complete `Cat`, including its `Animal` portion. |

Class types do not prescribe stack or heap storage. [COLI](ownership.md) chooses a valid representation while preserving class identity, access, and lifetime requirements.

## 2. Primitive Values in the Example

| Type | Value range |
| --- | --- |
| `u8` | Integers from 0 through 255. |
| `u32` | Integers from 0 through 4,294,967,295. |

An unsuffixed decimal literal is checked against its expected type in this subset. The assignment to `age: u32` therefore accepts `0`. The argument to `Cat.new(color: u8)` accepts `7` and rejects `256` rather than silently truncating it.

Type names do not themselves denote numeric values. In particular, `self.color = u8;` does not initialize a `u8` field with a value.

This establishes contextual integer literal checking for the example. Numeric promotion, arithmetic overflow, conversion syntax, and the complete primitive type set still require specification.

## 3. Field Typing and Lookup

Writing `self.field = expression` requires:

1. The field resolves to a declared or accessible inherited instance field.
2. Its visibility permits access at the use site.
3. The expression is assignable to the field's declared type.
4. The receiver permits the required write or initialization operation.
5. Construction and ownership rules permit the operation at that point.

Assignability includes class-subtype conversion under [inheritance semantics](inheritance.md). It does not authorize implicit narrowing, arbitrary cloning, or loss of a derived object's state.

An unknown field, an inaccessible field, or a type mismatch MUST produce an error. Inherited fields refer to their original base storage; declaring another accessible field with the same name MUST NOT silently hide it.

## 4. Visibility

| Declaration | Without `pub` | With `pub` |
| --- | --- | --- |
| Class | Private under module rules. | Public. |
| Field | Private under module rules. | Public. |
| Ordinary class method or associated function | Private under module rules. | Public. |
| Interface method | Public within the interface's accessibility. | Not required by the interface syntax. |
| `new` constructor | Public within the class's accessibility. | The same public constructor. |

Always-public `new` does not make its enclosing class public. Inheritance grants no extra access to private members and introduces no `protected` modifier.

The complete module privacy and export model remains to be formalized. It must preserve imported Rust visibility restrictions.

## 5. Constructor Signatures

For source-level type checking, a `new` declaration in class `C` has an associated callable contract with its declared parameter types and result `C`. Its initialization receiver is not an argument supplied by the caller.

For the starting example:

```text
Animal.new: () -> Animal
Cat.new: (color: u8) -> Cat
```

There is no virtual constructor lookup. `Cat.new` constructs a `Cat`; the presence of `Animal.new` alone does not supply an inherited `Cat.new`.

An ordinary method named differently may be a factory, but it does not acquire constructor initialization privileges or implicit public visibility.

## 6. Interface and Generic Contracts

The foundation already requires these distinctions:

- A class type such as `Animal` accepts its subclasses while preserving class-polymorphic behavior.
- An interface type without `dyn` denotes a static generic contract.
- `dyn Interface` denotes dynamic interface dispatch.
- A class implementing an interface must provide compatible public implementations of required methods.

This document does not yet define full generic inference, associated types, conflict resolution, interface composition, or substitutions involving complex Rust bounds. Stage 1 must choose a formal core and describe any deferred boundaries explicitly.

## 7. Required Validation Cases

| Case | Expected result |
| --- | --- |
| `Animal.new()` with its declared constructor. | Result has class contract `Animal`. |
| `Cat.new(7)`. | Result has class contract `Cat`; the integer fits `u8`. |
| `Cat.new(256)`. | Reject the out-of-range argument. |
| `self.color = color;` for a `color: u8` parameter. | Accept when initialization or mutation access is valid. |
| `self.color = u8;`. | Reject a type name used as the required numeric value. |
| Access an unknown field. | Reject with a source-level member diagnostic. |
| Omit `pub` on `new`. | Constructor remains public within class accessibility. |
| Use a subclass where its base is expected. | Accept when access, ownership, and lifetime requirements are met. |
