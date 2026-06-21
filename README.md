# Overview

This is a practice project – my first using Rust.

It contains the core code for a spreadsheet app and has the following features:
- Clean code, clear separation of concerns, highly modularized
  - Specifically, extending with a new formula type only requires one tiny change to the existing modules, namely the addition of the new type to a list
- Evaluation happens iteratively, not recursively, so stack overflow errors cannot happen
- Using dependency tracking, only the relevant parts of the spreadsheet are recalculated when a cell changes
  - Dependencies are adjusted depending on evaluated values. For example, for a formula of the form `IF(a,b,c)`, the dependencies are those of `a` plus those of `b` only if `a` evaluates to true and those of `c` only if `a` evaluates to false
- The dependency graph is maintained with range-based indexes, avoiding the need to ever iterate over all cells
