# BMSSP implementation analysis (Duan–Mao–Mao–Shu–Yin, arXiv:2504.17033v2)

This note translates the paper sources in `bmssp/` into implementable details for the **bounded multi-source shortest path (BMSSP)** subroutine and how it is used to solve directed **single-source shortest paths (SSSP)**.

Primary sources (LaTeX):
- `bmssp/preliminary.tex` (labels, completeness, tie-breaking)
- `bmssp/rebundle.tex` (**FindPivots**)
- `bmssp/data_structure.tex` (partition data structure `𝒟`: Insert / BatchPrepend / Pull)
- `bmssp/main_result.tex` (**BMSSP** + **BaseCase**, correctness/intuition)

## 0. What BMSSP is (and what it is not)

BMSSP is **not** a stand-alone shortest path algorithm that you can call once to get all distances (unless you call it at the top level exactly as the paper does). BMSSP is a **bounded, multi-source** subroutine designed to avoid fully sorting a large “frontier” of tentative distances, which is where Dijkstra spends Θ(n log n).

- Input to BMSSP: a *set of sources* `S` (some may be incomplete) and a *bound* `B` (called `\expectB` in the paper).
- Output from BMSSP: a *new boundary* `B' ≤ B` and a *set of vertices* `U` that are guaranteed to be **complete** at the end of the call.

At the very top level, the paper solves SSSP by calling:

```
l = ceil(log n / t)
BMSSP(l, +∞, {s})
```

and then all vertices become complete, yielding all distances.

## 1. Graph model and global state

### 1.1 Graph assumptions

The analysis assumes:
- Directed graph `G=(V,E)`, non-negative weights `w(u,v) ≥ 0`.
- **Constant in/out-degree** (paper reduces general graphs to constant-degree using a standard transformation).

Implementation choices:
- If your graphs already have small degree, you can skip the transformation.
- If you need the paper’s asymptotics, apply the constant-degree transformation (see §8.2).

### 1.2 Global labels: `d̂[v]` and “complete”

The algorithm maintains a global **tentative label** `d̂[v]` (paper: `\hat{d}[v]`) for every vertex:
- Invariant: `d̂[v] ≥ d(v)` (true shortest path distance; paper: `d(v)`).
- Initialization: `d̂[s]=0`, `d̂[v]=+∞` for `v≠s`.
- Only operation allowed to change `d̂` is **relaxation** along an edge `(u,v)`:
  - `d̂[v] ← min(d̂[v], d̂[u] + w(u,v))` (but see tie-breaking and the crucial `≤` condition in §2.3).

Definition:
- A vertex `v` is **complete** iff `d̂[v] == d(v)`.
- The algorithm never directly knows `d(v)`; instead it *proves* completeness for the vertices it returns from subroutines. In code, represent this with a boolean `complete[v]` that you set when BMSSP says a vertex is complete.

### 1.3 Predecessor pointers

The algorithm also maintains predecessor pointers `Pred[v]` (paper: `\textsc{Pred}[v]`) so current labels define a shortest-path tree for the current ordering:
- When you perform a relaxation that updates the label of `v` to come from `u`, set `Pred[v]=u`.
- This is required for:
  - Tie-breaking / total order on paths (§2).
  - Building the forest in `FindPivots` (§4.4).
  - Optionally reconstructing paths after the algorithm finishes.

## 2. Total order / tie-breaking (needed for correctness with 0-weight edges)

The paper assumes “all paths we obtain have different lengths” (Assumption 2.1 in `bmssp/preliminary.tex`) and then explains how to enforce it by defining a total order on paths.

You should implement **labels** as an “extended” value, not just a numeric distance.

### 2.1 Why you need it

Two places break without a strict total order:
- The predecessor structure may stop being a tree if multiple equal-length paths exist.
- The paper relies on an equality-aware relaxation condition (`≤`) so an edge “relaxed” at a lower recursion level can be reused at higher levels (see remark in `bmssp/main_result.tex`).

The constant-degree transformation introduces **0-weight edges**, making ties common unless you tie-break.

### 2.2 A practical label type

The paper’s total order is lexicographic over the path tuple:
`⟨length, hops, v_end, v_{end-1}, …, s⟩`.

In code, you can implement an equivalent O(1) comparison by storing per-vertex:
- `dist`: numeric weight sum (exact if possible)
- `hops`: number of edges on the path
- `pred`: predecessor vertex id (for same-endpoint comparisons)
- and using the endpoint id itself when comparing labels of different endpoints.

Concretely:
- Comparing labels of different vertices `u != v`: compare `(dist, hops, endpoint_id)`.
- Comparing two candidate labels for the same vertex `v`: compare `(dist, hops, pred_id)` (because endpoint is equal).

This matches the paper’s “it suffices to compare endpoints” / “it suffices to compare predecessors” discussion.

### 2.3 Relaxation condition must be `candidate ≤ current`

In the paper, the relaxation condition is always written as:

`d̂[u] + w(u,v) ≤ d̂[v]`

not `<`. Implement this as:

```
candidate = label_add(dhat[u], w(u,v), pred=u)
if candidate <= dhat[v]:
    dhat[v] = candidate
    Pred[v] = u
```

Where `<=` is the total order on labels (not raw float `<=`).

If you use raw distances and `<` only, you can lose necessary re-insertions and the recursion invariants can fail.

## 3. BMSSP contract (what you must implement)

BMSSP is defined in Lemma “Bounded Multi-Source Shortest Path” (`bmssp/main_result.tex`, `\label{lemma:bmssp}`).

### 3.1 Inputs

`BMSSP(layer, B, S)` where:
- `layer ∈ [0, ceil(log n / t)]`
- `|S| ≤ 2^(layer * t)`
- `B` is a strict upper bound over current labels on sources (`B > max_{x∈S} d̂[x]`)
- **Key dependency condition**: for every incomplete vertex `v` with `d(v) < B`, the shortest path to `v` visits some **complete** vertex in `S`.

### 3.2 Outputs

Returns `(B', U)` with:
- `B' ≤ B`
- At end of the call, **all vertices in `U` are complete**
- `U` contains **every vertex `v` with `d(v) < B'` whose shortest path visits some vertex in `S`**
- Either:
  - **successful execution**: `B' = B` (BMSSP fully resolves the bound), or
  - **partial execution**: `B' < B` and `|U| = Θ(k * 2^(layer * t))` (BMSSP stops after doing a “chunk” of work).

The “boundary” `B'` is the algorithm’s way to say: “I proved correctness up to here, but I’m stopping early because too much work would be required to go all the way to `B`.”

## 4. Subroutine: `FindPivots(B, S)`

Source: `bmssp/rebundle.tex` (Lemma `\label{lemma:findpivots}` + Algorithm “Finding Pivots”).

### 4.1 Purpose

`FindPivots` shrinks the source set so the recursion can’t degenerate into fully sorting a large frontier.

Intuition:
- Run `k` Bellman-Ford-like relaxation “steps” starting from `S`.
- Any vertex whose shortest path from the *first complete source on its shortest path* uses ≤ `k-1` edges becomes complete during those steps.
- The remaining unresolved region must be “heavy”: it forms large shortest-path subtrees rooted at a small subset of sources. Those roots are the **pivots**.

### 4.2 Inputs / assumption

`FindPivots(B, S)` assumes:
- for every incomplete `v` with `d(v) < B`, its shortest path visits some complete vertex in `S`.

### 4.3 Outputs

It returns:
- `W` (paper `\pivotU`): a visited set of size `O(k|S|)` (or it early-aborts and still returns `O(k|S|)` due to constant degree)
- `P` (paper `\pivotP`): a subset of `S` with `|P| ≤ |W|/k`

Guarantee (important for BMSSP):
For every vertex `x` that BMSSP ultimately cares about under bound `B`:
- either `x ∈ W` and `x` is complete after the `k` relaxations, or
- the shortest path to `x` visits some **complete** vertex in `P`.

### 4.4 Algorithm details (implementable)

Pseudocode from the paper (with explicit sets):

1. Initialize:
   - `W = S`
   - `W_0 = S`
2. For `i=1..k`:
   - `W_i = ∅`
   - For each edge `(u,v)` with `u ∈ W_{i-1}`:
     - If `relax(u,v)` succeeds (in the `≤` sense; §2.3) and the new label of `v` is `< B`, then add `v` to `W_i`.
   - `W = W ∪ W_i`
   - If `|W| > k|S|`, then **early return**: `P = S`, return `(P, W)`.
3. Build a directed forest:
   - `F = {(u,v) ∈ E : u,v ∈ W and d̂[v] = d̂[u] + w(u,v)}`
   - Under the unique-label assumption, each `v` has at most one such parent `u` in `W`, so `F` is a forest.
   - In an actual implementation, it’s simpler and more robust to define `F` as: `F = {(Pred[v], v) : v ∈ W and Pred[v] ∈ W}`.
4. Compute subtree sizes in `F`, and output:
   - `P = {u ∈ S : u is a root of a tree with ≥ k vertices in F}`
   - Return `(P, W)`.

Implementation notes:
- Maintain `in_W[v]` to avoid pushing duplicates; count `|W|` accurately.
- Early-return threshold `k|S|` depends on constant degree to keep work bounded.
- For subtree sizes:
  - Build children lists for `F` on the induced set `W`.
  - Identify roots as vertices in `S` with no parent in `F` (or parent not in `W`).
  - Postorder count sizes.

## 5. Data structure: `𝒟` (Insert / BatchPrepend / Pull)

Source: `bmssp/data_structure.tex` (Lemma `\label{lemma:partition}`).

BMSSP needs a data structure that can “partially sort”:
- It repeatedly pulls up to `M = 2^((layer-1) t)` smallest-key items,
- without maintaining a full priority queue over all frontier elements.

### 5.1 Required operations (semantic contract)

The data structure stores key/value pairs `(vertex, label)`, keeping the **smallest label per key**.

Operations:
- `Initialize(M, B)`
- `Insert(key, value)`:
  - Insert/update in amortized `O(max(1, log(N/M)))`.
- `BatchPrepend(list L)`:
  - Precondition: every value in `L` is smaller than any value currently in `𝒟`.
  - Insert all of them in amortized `O(L * max(1, log(L/M)))`.
- `Pull()`:
  - Remove and return up to `M` keys with the smallest values (if fewer exist, return all).
  - Also return a boundary `x` such that:
    - if the structure is empty afterward: `x = B`,
    - else `max(pulled_values) < x ≤ min(remaining_values)`.

BMSSP interprets this returned `x` as the next recursive bound `B_i`.

### 5.2 Paper’s concrete implementation (block lists + BST)

The paper’s implementation is designed to satisfy the amortized bounds:
- Maintain two block sequences:
  - `D0`: elements from `BatchPrepend` only (prepended in decreasing “time”, so globally smallest first)
  - `D1`: elements from `Insert` operations
- Each block holds up to `M` elements (linked list; not necessarily internally sorted).
- Blocks are kept in sorted order *by block ranges*:
  - For blocks `B_i` before `B_j`, any value in `B_i` is `≤` any value in `B_j`.
- For blocks in `D1`, maintain an upper bound per block, and put those bounds in a balanced BST (e.g., red-black tree) so Insert can find a block by value in `O(log(N/M))`.

Key duplication handling:
- Keep an index/map `key -> (which list, which block, pointer)` and the current stored value.
- On Insert/BatchPrepend of an existing key:
  - if new value is not smaller, ignore
  - else delete old pair (O(1) from linked list if you have the pointer), then insert the new pair.

Split on overflow (in `D1`):
- If a `D1` block exceeds `M`, select the median and partition into 2 blocks of size ≤ `ceil(M/2)` (median selection is `O(M)`).
- Update BST upper bounds.

BatchPrepend:
- If `L ≤ M`, create one new block and prepend to `D0`.
- Else partition `L` into `O(L/M)` blocks, each ≤ `ceil(M/2)`, using repeated median partitioning (conceptually quicksort-style on block boundaries) in `O(L log(L/M))`, then prepend those blocks to `D0` in order.

Pull:
- Collect a prefix of blocks from `D0` and `D1` until you have ≥ `M` candidates or the lists end.
- If total candidates ≤ `M`, return them all and `x=B`.
- Else select the `M` smallest from the candidate set in `O(M)` (selection algorithm), delete them, and set `x` to the smallest remaining value in the candidate set (or the next block lower bound).

### 5.3 Practical implementation shortcut (correctness-first)

If you only need correctness (not the paper’s asymptotics), you can replace `𝒟` with:
- A binary heap / pairing heap / BTreeMap keyed by value.
- Implement `BatchPrepend(L)` as `for (k,v) in L { Insert(k,v) }`.
- Implement `Pull()` as pop up to `M` items.

This will be simpler but moves you back toward `O(m log n)` behavior.

## 6. Base case: `BaseCase(B, S={x})`

Source: `bmssp/main_result.tex` (Algorithm “Base Case of BMSSP”).

### 6.1 Contract

Inputs:
- `S` is a singleton `{x}`
- `x` is complete
- for every incomplete `v` with `d(v) < B`, the shortest path to `v` visits `x`

Output:
- If fewer than `k+1` vertices are found under bound `B`: return `(B, U0)`
- Else return `(B', U)` where:
  - `B'` is the maximum label among the `k+1` extracted vertices
  - `U` is the set of the `k` extracted vertices with label `< B'`

### 6.2 Implementation details

This is a bounded Dijkstra from `x` that stops after extracting `k+1` vertices:
- Use a min-heap keyed by the extended label (§2).
- Standard “ignore stale entries” pattern works (no need for DecreaseKey).
- Only push a neighbor `v` if its new label `< B` (strictly).
- Relaxation uses the `≤` condition (extended ordering).

Mark all returned vertices as `complete`.

## 7. Main BMSSP recursion

Source: `bmssp/main_result.tex` (Algorithm “Bounded Multi-Source Shortest Path”).

### 7.1 Parameters

Global parameters:
- `k = floor(log(n)^(1/3))`
- `t = floor(log(n)^(2/3))`
- `l = ceil(log n / t)`

Per call:
- `layer` decreases until 0
- `M = 2^((layer-1) * t)` (only for `layer>0`)
- Work budget for partial execution: stop once `|U|` reaches `k * 2^(layer * t)`

### 7.2 High-level shape

For `layer=0`: run `BaseCase`.

For `layer>0`:
1. Shrink sources using `FindPivots(B,S)` → `(P,W)`
2. Initialize `𝒟` with `M` and insert all pivots `P` keyed by current `d̂[p]`
3. Repeat:
   - `Pull()` gives a subset `S_i` of at most `M` “smallest” keys and a separating bound `B_i`
   - Recurse: `BMSSP(layer-1, B_i, S_i)` → `(B'_i, U_i)`
   - Relax outgoing edges from `U_i` and insert affected neighbors into `𝒟`
   - `BatchPrepend` values that are now known to be `< B_i` (so they must go before the rest)
   - Stop if `𝒟` empty (success) or budget exceeded (partial)
4. Before returning, add the subset of `W` whose labels are `< B'` (they are guaranteed complete by the FindPivots lemma + the boundary logic)

### 7.3 Exact loop logic (careful about ranges)

On iteration `i`:
- `Pull()` returns `(B_i, S_i)`
  - `S_i` are the keys with smallest values
  - all remaining keys in `𝒟` have value `≥ B_i`
- Recurse with bound `B_i`:
  - returns `(B'_i, U_i)` where `B'_i ≤ B_i`
- For each relaxed edge `(u,v)` with `u∈U_i`:
  - Let `new = d̂[u] + w(u,v)` (extended add)
  - If `new` is in `[B_i, B)`:
    - `Insert(v, new)` directly (these values belong after the “pulled frontier”)
  - Else if `new` is in `[B'_i, B_i)`:
    - Add `(v,new)` to a temporary list `K`
- After scanning edges:
  - `BatchPrepend(K ∪ { (x, d̂[x]) : x ∈ S_i and d̂[x] ∈ [B'_i, B_i) })`

The second term re-inserts “still relevant” sources from `S_i` if the recursive call was partial and they remain in the unresolved interval.

### 7.4 Return value

When the loop ends:
- If `𝒟` empty, the call is successful and returns `B'=B`.
- If budget exceeded, returns `B' < B` and `|U| = Θ(k*2^(layer*t))`.

In the paper pseudocode, they return:
`B' = min(B'_i, B)` and then
`U ← U ∪ {x ∈ W : d̂[x] < B'}`.

Implementation notes:
- Track the “current” `B'_i` from the last recursion; initialize `B'_0 = min_{p∈P} d̂[p]` (or `B` if `P` empty).
- If the loop never runs (`𝒟` empty immediately), you still return `B'=B` and add `W` elements `< B` as appropriate.

## 8. Using BMSSP to solve full SSSP

### 8.1 Top-level procedure

To compute SSSP from `s`:
1. (Optional) transform to constant-degree graph.
2. Initialize:
   - `d̂[s]=0`, others `+∞`
   - `complete[s]=true`, others `false`
3. Choose parameters `k,t,l` as above.
4. Call `BMSSP(l, +∞, {s})`.
5. All vertices are now complete; return `d̂[v].dist` (raw component) and `Pred` if you want paths.

### 8.2 Constant-degree transformation (if you need it)

From `bmssp/preliminary.tex`:
- Replace each vertex `v` with a 0-weight directed cycle of “ports”, one per incident edge.
- Replace each original edge `(u,v)` with an edge from `u`’s outgoing port to `v`’s incoming port with weight `w(u,v)`.
- This yields max in/out-degree 2 and preserves shortest path distances.

If you apply this:
- Your “real” distances are on the port nodes; map them back to original vertices by taking the minimum over the ports representing that vertex.

### 8.3 When is Dijkstra’s priority queue a dominant cost?

The paper’s headline improvement is about reducing the cost of **global sorting** inherent in classic Dijkstra implementations. In practice, the priority queue becomes a dominant cost when:

- **You compute SSSP to many/all vertices** (not just a single target), so you perform roughly `Θ(m)` relaxations and `Θ(m)` priority-queue insertions plus `Θ(n)` extract-min operations.
- **The graph is large and sparse** (e.g., `m = Θ(n)`), so the algorithm performs many queue operations relative to the actual arithmetic per edge.
- **Edge relaxation is cheap** (simple integer add/compare) and neighbor iteration is cheap (in-memory adjacency lists), making heap maintenance stand out.
- **You run SSSP many times** (many different sources, centrality/analytics, repeated “what-if” runs), so the `log n` factor is paid repeatedly.
- **A* does not help** because you do not have (or cannot use) a strong admissible heuristic, or because the workload is inherently “all targets” rather than “one goal”.
- **Weights are general nonnegative values**, so you cannot switch to specialized queues that avoid `log n` in common cases (e.g., `0–1 BFS` for `{0,1}` weights, or Dial’s algorithm for small integer weights).

BMSSP’s role in the full SSSP algorithm is to avoid maintaining a fully sorted global frontier by:
- Working under a moving **bound** `B` and proving completeness for a batch of vertices below it.
- Using the partition structure `𝒟` (Insert/BatchPrepend/Pull) to perform only the amount of “ordering” needed to make progress, rather than a full priority-queue ordering at all times.

This is why the paper frames the result as “breaking the sorting barrier”: it reduces the amount of priority-queue-like work needed to advance the shortest-path boundary.

## 9. Implementation checklist (minimal, in dependency order)

1. Define a `Label` type implementing:
   - total order (`Ord`)
   - “addition” of a weight plus predecessor bookkeeping
2. Implement `relax(u,v,w)` with the `≤` rule.
3. Implement `FindPivots(B,S)`:
   - k-step relaxation with `|W|` cap
   - forest from `Pred` and subtree size computation
4. Implement the partition queue `𝒟`:
   - correctness-first heap version (fast to build), or
   - block-list version from the paper (for the stated asymptotic bound)
5. Implement `BaseCase(B,{x})`.
6. Implement `BMSSP(layer,B,S)` exactly with:
   - `FindPivots`
   - `Pull`/recurse/relax/Insert-or-K/BatchPrepend loop
   - return boundary + marking `complete`
7. Validate on random graphs by comparing distances to standard Dijkstra (same `Label.dist`).

## 10. Common pitfalls

- Using raw floating-point `f64` comparisons can break the “comparison-addition model” assumptions; if you must use floats, you may need a deterministic tie-break that does not rely on exact equality of sums.
- Do not replace `≤` by `<` in relaxation checks; the paper explicitly requires equality handling for reuse across recursion levels.
- Ensure `BatchPrepend`’s precondition holds: only prepend values proven `< current Pull boundary`.
- Remember: the paper assumes all vertices are reachable from `s`. In a general library, keep unreachable vertices at `+∞` and treat “complete” carefully.
