# Overview

This is a practice project – my first using Rust.

It contains the core code for a spreadsheet app and has the following features:
- Clean code, clear separation of concerns, highly modularized
  - Specifically, extending with a new formula type only requires one tiny change to the existing modules, namely the addition of the new type to a list
- Evaluation happens iteratively, not recursively, so stack overflow errors cannot happen
- Using dependency tracking, only the relevant parts of the spreadsheet are recalculated when a cell changes
  - Dependencies are adjusted depending on evaluated values. For example, for a formula of the form `IF(a,b,c)`, the dependencies are those of `a` plus those of `b` only if `a` evaluates to true and those of `c` only if `a` evaluates to false
- The dependency graph is maintained with range-based indexes, avoiding the need to ever iterate over all cells

# Design details

The most important object types are Spreadsheet and Cell. Cells have a unique address, represented by two u32s, and the cells are collected in a spreadsheet's CellMap, which allows for efficient lookup by that address. The cells that in a spreadsheet UI are empty are not represented by a Cell object.

The central properties of a Cell are its formula, which is kept in both the form of the raw string input and in parsed form, and its value. In addition, a Cell has parents and children. Cell A is the parent of Cell B, and B is the child of A, if the value of Cell A depends on the value of Cell B. Note that this naming follows the propagation direction: when a child's value changes, that change propagates up to its parents, which must then re-evaluate. Finally, a cell A has a collection of child rectangles, which represent the areas of cells — both those that are represented as a Cell object and those that are not — on which A depends. For example, if cell A has the formula `SUM(B1:C2)`, then the rectangle consisting of B1, B2, C1, and C2 is the (only) child rectangle of A, and if B1, B2, and C1 are empty, then C2 is the only child of A.

The previous paragraph is not entirely accurate: Only so-called "independent cells" have all the six mentioned properties. This is because of dynamic arrays. For example, with the formula `SEQUENCE(3,1)` in cell A1, the cells A1, A2, and A3 are given the values 1, 2, and 3, respectively. A1:A3 is then the "spill area" of cell A1, A1 is the "anchor", and A2 and A3 are the "dependent cells" of A1. They have no raw formula, parsed formula, child rectangles, or children, just a value and parents. A spill area is always a rectangle, and the anchor cell is always that rectangle's top left cell.

In many cases, the child rectangles of a cell can be determined based just on its formula. That is, for example, the case when the formula is "SUM(B1, C2)". But that is not always the case. Consider the formula "IF(A1, B2, C3)", which evaluates to B2 if A1 is true, and to C3 if A1 is false. The rectangle containing just A1 is definitely a child rectangle, but the rectangle containing just B2 is only a child rectangle if A1 is true. Child rectangles can therefore be "discovered" during evaluation. Each formula has a method that returns its initial child rectangles, i.e., those that can be determined based on the formula alone.

A cell can be "volatile". This means that it is always re-evaluated when the user changes anything in the spreadsheet. An example of a volatile cell is one with the formula "RANDBETWEEN(1, 10)". Like the cell's address, this data is not stored as a property of the cell itself, but instead centrally in collections that belong to the spreadsheet. The spreadsheet has two further collections: First, a ParentLookupTree which is a reverse spatial index that makes it efficient to find the parents of a new cell. And second, a SpillOwnershipMap that keeps track of which cells are the anchors of which spill areas. Together, these collections ensure that there is never a need to iterate over all the cells of a spreadsheet.

When all goes well, each Cell will get a value that is a number, a boolean, or a string. There are several ways that things can go "badly":

- A Cell's raw formula does not parse. Then the value is an error.
- The input to the calculation of a Cell's value is not of the correct type. For example, the formula is "SUM(A1,B2)", but A1's value is a string. Also in this case, the value is an error.
- The parent-child graph contains a cycle. In this case, the cells that are part of a cycle or depend on one have the "None" value.
- A dynamic array's "desired" spill area is not free - (partially) occupied by either an independent Cell or the spill area of another anchor Cell. In this case, the value is also an error. The SpillOwnershipMap keeps track of both actual spill areas and blocked ones, so the latter can become actual if later unblocked.
- A dynamic array's "desired" spill area would extend beyond the right or the bottom edge of the spreadsheet. Again, this leads to an error value.

## The evaluation loop

The user can modify the spreadsheet in just one way: by changing the raw formula of a cell. If they input a non-empty formula for a cell address that previously had no Cell object, that operation creates a Cell. If there was one, they modify it. And if they enter an empty formula for a cell address that had a Cell object, that operation deletes the Cell object.

Such a change triggers a re-evaluation of the relevant parts of the spreadsheet. In the simplest cases, this proceeds in two clearly separated phases. First, the parent-child graph is updated, and the Cell in question together with its parents and their parents etc. are reset. That is, their value is set to None, their collection of child rectangles is set to the initial child rectangles, and if they have a spill area of dependent cells, they are removed. The same happens to any volatile Cells. Then the reset Cells that are ready to be evaluated (i.e., those that have no children with the value None) are identified and placed in an evaluation queue. Then one at a time, they are evaluated based on their parsed formula and the values of their children and assigned the resulting value. Each time a Cell is evaluated, each parent is checked for evaluation readiness, with a successful check resulting in addition to the evaluation queue. When the evaluation queue is empty, there may be some Cells with the None value left over; they are the ones affected by circularity.

When a Cell is attempted evaluated, it is because all of its *already-identified* children have a value. As explained above, more children may be identified during the evaluation attempt. If these all have a value, the evaluation proceeds, and the Cell is provided with a value as well as an updated set of child rectangles and children. If not, just the child rectangles and the children are updated. The Cell will then be re-checked for evaluation readiness each time one of its newly discovered children is assigned a value.

The clear separation into two phases fails when the result of the evaluation of a Cell is the creation of a spill area. In that case the "phase 1" operations happen for each of the created dependent cells while "phase 2" is in progress, so the two processes get entangled. This means that the main mechanism to detect circularity (that Cells are reset to the value None, which simply doesn't change for a given Cell if it is affected by circularity) does not suffice by itself. The problem is that a Cell may be evaluated because all its *already existing* children have a value, but more children are added later, and the Cell has to be re-evaluated. Under particularly unfortunate circumstances, this can result in oscillating value assignments that would never stabilize. For this reason, the evaluation queue is designed to not allow the same Cell to be added more than 100 times.  

When the evaluation queue is empty, it is checked whether any blocked spill areas have become free. If so, the relevant anchor Cells are reset and added to the queue.

## Invariants

The following invariants are maintained throughout the lifetime of the spreadsheet:

1. Bidirectional child/parent consistency: A ∈ B.children ↔ B ∈ A.parents. Every attach/detach operation maintains both sides.
2. ParentLookupTree mirrors ChildRectangles: The tree contains (address, rectangle) exactly when the Cell with that address is Independent and rectangle ∈ the Cells ChildRectangles. The methods add_to_parent_lookup_tree and remove_from_parent_lookup_tree are called in lockstep with every mutation to ChildRectangles.
3. Active spill ownership is exclusive: For every Active entry (owner, rectangle), every cell in rectangle other than owner is a Dependent cell of that owner. No two Active claims overlap (enforced by a call to the method spill_area_is_free before inserting Active).
4. Blocked claims retain no dependent cells: Blocked owners never create dependent cells; their claim is just a record that lets them re-compete if the active owner is removed.
