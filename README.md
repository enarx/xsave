[![Workflow Status](https://github.com/enarx/xsave/workflows/test/badge.svg)](https://github.com/enarx/xsave/actions?query=workflow%3A%22test%22)
[![Average time to resolve an issue](https://isitmaintained.com/badge/resolution/enarx/xsave.svg)](https://isitmaintained.com/project/enarx/xsave "Average time to resolve an issue")
[![Percentage of issues still open](https://isitmaintained.com/badge/open/enarx/xsave.svg)](https://isitmaintained.com/project/enarx/xsave "Percentage of issues still open")
![Maintenance](https://img.shields.io/badge/maintenance-activly--developed-brightgreen.svg)

# xsave

This crate contains a practical implementation of the x86 xsave semantics.

We do not intend to support all possible variations of the instructures,
nor do we intend to calculate the size of the xsave area dynamically.
Instead, our practical strategy will overallocate the size of the xsave
area so that we get a constant size for the struct. This allows for
substantially easier embedding in other contexts.

For example, clearing the extended CPU state is a simple:

```rust
use xsave::XSave;

XSave::default().load();
```

Likewise, you can save and restore the extended CPU state like this:

```rust
use xsave::XSave;

let mut xsave = XSave::default();
xsave.save();
xsave.load();
```

License: Apache-2.0
