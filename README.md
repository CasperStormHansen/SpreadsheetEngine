# Overview

This is a practice project - my first using Rust.

It contains the core code for a spreadsheet app and has the following features:
- Clean code, clear separation of concerns, highly modularized
  - Specifically, extending with a new formula type only requires one tiny change to the existing modules, namely addition of the new type to a list
- Using dependency tracking, only the relevant parts of the spreadsheet are recalculated when a cell changes
- Evaluation happens iteratively, not recursively, so stack overflow errors cannot happen
