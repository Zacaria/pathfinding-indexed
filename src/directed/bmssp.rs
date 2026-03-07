//! Compute single-source shortest paths using the BMSSP-based algorithm from
//! “Breaking the Sorting Barrier for Directed Single-Source Shortest Paths”
//! (Duan–Mao–Mao–Shu–Yin, arXiv:2504.17033).
//!
//! This module exposes an SSSP routine that follows the structure of the paper’s
//! `BMSSP` recursion (bounded multi-source shortest path).
//!
//! ## Notes
//!
//! - Edge weights must be non-negative.
//! - The original paper’s time bound assumes constant in/out degree. This
//!   implementation applies the port-graph transformation internally (which
//!   increases preprocessing time and memory) to preserve the API while matching
//!   the paper’s assumption, but the asymptotic bound is still **not** guaranteed
//!   for arbitrary inputs.

use num_traits::Zero;
use std::cmp::Ordering;
use std::cmp::Reverse;
use std::collections::{BTreeMap, BinaryHeap, VecDeque};

type Bound<C> = Option<Label<C>>;

#[derive(Clone, Copy, Debug)]
struct Label<C> {
    cost: C,
    hops: usize,
    node: usize,
    pred: usize,
}

impl<C: Ord + Copy> PartialEq for Label<C> {
    fn eq(&self, other: &Self) -> bool {
        self.cost == other.cost
            && self.hops == other.hops
            && self.node == other.node
            && self.pred == other.pred
    }
}

impl<C: Ord + Copy> Eq for Label<C> {}

impl<C: Ord + Copy> PartialOrd for Label<C> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<C: Ord + Copy> Ord for Label<C> {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.cost, self.hops, self.node, self.pred)
            .cmp(&(other.cost, other.hops, other.node, other.pred))
    }
}

#[derive(Clone, Copy, Debug)]
struct NodeState<C> {
    label: Option<Label<C>>,
    pred: usize,
    complete: bool,
}

impl<C> Default for NodeState<C> {
    fn default() -> Self {
        Self {
            label: None,
            pred: usize::MAX,
            complete: false,
        }
    }
}

struct Params {
    k: usize,
    t: usize,
    l: usize,
}

struct MarkSet {
    marks: Vec<u32>,
    epoch: u32,
}

impl MarkSet {
    fn new(len: usize) -> Self {
        Self {
            marks: vec![0; len],
            epoch: 1,
        }
    }

    fn next_epoch(&mut self) -> u32 {
        self.epoch = self.epoch.wrapping_add(1);
        if self.epoch == 0 {
            self.marks.fill(0);
            self.epoch = 1;
        }
        self.epoch
    }

    fn is_marked(&self, idx: usize, epoch: u32) -> bool {
        self.marks[idx] == epoch
    }

    fn mark(&mut self, idx: usize, epoch: u32) {
        self.marks[idx] = epoch;
    }
}

struct Scratch {
    in_w: MarkSet,
    extracted: MarkSet,
    out_set: MarkSet,
    parent_in_w: Vec<usize>,
    subtree_size: Vec<usize>,
    children: Vec<Vec<usize>>,
}

impl Scratch {
    fn new(len: usize) -> Self {
        Self {
            in_w: MarkSet::new(len),
            extracted: MarkSet::new(len),
            out_set: MarkSet::new(len),
            parent_in_w: vec![usize::MAX; len],
            subtree_size: vec![0; len],
            children: vec![Vec::new(); len],
        }
    }
}

struct IndexedContext<C: Ord + Copy> {
    states: Vec<NodeState<C>>,
    scratch: Scratch,
    queue_pool: Vec<PartitionQueue<C>>,
}

impl<C: Ord + Copy> IndexedContext<C> {
    fn new(number_of_nodes: usize, start: usize, start_label: Label<C>) -> Self {
        let mut states = Vec::with_capacity(number_of_nodes);
        states.resize_with(number_of_nodes, NodeState::default);
        if let Some(state) = states.get_mut(start) {
            state.label = Some(start_label);
            state.pred = usize::MAX;
            state.complete = true;
        }
        Self {
            states,
            scratch: Scratch::new(number_of_nodes),
            queue_pool: Vec::new(),
        }
    }

    const fn len(&self) -> usize {
        self.states.len()
    }

    fn state(&self, idx: usize) -> &NodeState<C> {
        &self.states[idx]
    }

    fn state_mut(&mut self, idx: usize) -> &mut NodeState<C> {
        &mut self.states[idx]
    }

    fn take_queue(&mut self, max_pull: usize, upper_bound: Bound<C>) -> PartitionQueue<C>
    where
        C: Ord + Copy,
    {
        if let Some(mut queue) = self.queue_pool.pop() {
            queue.reset(max_pull, upper_bound, self.len());
            queue
        } else {
            PartitionQueue::new(max_pull, upper_bound, self.len())
        }
    }

    fn recycle_queue(&mut self, queue: PartitionQueue<C>) {
        self.queue_pool.push(queue);
    }
}

const fn log2_floor(n: usize) -> usize {
    if n <= 1 {
        0
    } else {
        (usize::BITS as usize - 1) - n.leading_zeros() as usize
    }
}

const fn log2_ceil(n: usize) -> usize {
    if n <= 1 {
        0
    } else {
        let f = log2_floor(n);
        if n.is_power_of_two() { f } else { f + 1 }
    }
}

const fn pow2_sat(exp: usize) -> usize {
    if exp >= (usize::BITS as usize) {
        usize::MAX
    } else {
        1usize << exp
    }
}

const fn cube_root_floor(n: usize) -> usize {
    let mut x = 0usize;
    loop {
        let next = x + 1;
        if next.saturating_mul(next).saturating_mul(next) > n {
            return x;
        }
        x = next;
    }
}

fn bound_min<C: Ord + Copy>(a: Bound<C>, b: Bound<C>) -> Bound<C> {
    match (a, b) {
        (None, x) | (x, None) => x,
        (Some(a), Some(b)) => Some(a.min(b)),
    }
}

fn bound_lt<C: Ord + Copy>(a: Label<C>, b: Bound<C>) -> bool {
    b.is_none_or(|b| a < b)
}

fn in_range<C: Ord + Copy>(x: Label<C>, lo: Bound<C>, hi: Bound<C>) -> bool {
    let ge_lo = lo.is_some_and(|lo| x >= lo);
    let lt_hi = hi.is_none_or(|hi| x < hi);
    ge_lo && lt_hi
}

fn label_add<C>(ctx: &IndexedContext<C>, u_idx: usize, v_idx: usize, w: C) -> Label<C>
where
    C: Zero + Ord + Copy,
{
    let u = ctx.state(u_idx).label.unwrap();
    Label {
        cost: u.cost + w,
        hops: u.hops + 1,
        node: v_idx,
        pred: u_idx,
    }
}

fn try_relax<C>(ctx: &mut IndexedContext<C>, v_idx: usize, candidate: Label<C>) -> bool
where
    C: Ord + Copy,
{
    let state = ctx.state_mut(v_idx);
    let improve = state.label.is_none_or(|cur| candidate <= cur);
    if improve {
        state.label = Some(candidate);
        state.pred = candidate.pred;
    }
    improve
}

#[derive(Clone, Copy, Debug)]
struct Location {
    block_id: usize,
    index: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BlockKind {
    D0,
    D1,
}

struct Block<C> {
    items: Vec<usize>,
    max: Label<C>,
    max_key: usize,
    kind: BlockKind,
}

#[derive(Clone, Copy)]
struct EpochOption<T> {
    epoch: u32,
    value: Option<T>,
}

impl<T: Copy> EpochOption<T> {
    const fn get(&self, epoch: u32) -> Option<T> {
        if self.epoch == epoch {
            self.value
        } else {
            None
        }
    }

    const fn set(&mut self, epoch: u32, value: T) {
        self.epoch = epoch;
        self.value = Some(value);
    }

    const fn clear(&mut self) {
        self.epoch = 0;
        self.value = None;
    }
}

struct PartitionQueue<C: Ord + Copy> {
    max_pull: usize,
    max_block: usize,
    upper_bound: Bound<C>,
    len: usize,
    blocks: Vec<Option<Block<C>>>,
    d0: VecDeque<usize>,
    d1: BTreeMap<(Label<C>, usize), usize>,
    epoch: u32,
    scratch_epoch: u32,
    best: Vec<EpochOption<Label<C>>>,
    loc: Vec<EpochOption<Location>>,
    scratch: Vec<EpochOption<Label<C>>>,
}

impl<C> PartitionQueue<C>
where
    C: Ord + Copy,
{
    // Heuristic caps to reduce D1 churn for tiny pulls.
    const MIN_PULL: usize = 4;
    const EXTRA_BLOCK: usize = 64;

    fn new(max_pull: usize, upper_bound: Bound<C>, node_count: usize) -> Self {
        let max_pull = max_pull.max(Self::MIN_PULL);
        let max_block = max_pull
            .saturating_mul(2)
            .min(max_pull.saturating_add(Self::EXTRA_BLOCK));
        Self {
            max_pull,
            max_block,
            upper_bound,
            len: 0,
            blocks: Vec::new(),
            d0: VecDeque::new(),
            d1: BTreeMap::new(),
            epoch: 1,
            scratch_epoch: 1,
            best: vec![
                EpochOption {
                    epoch: 0,
                    value: None
                };
                node_count
            ],
            loc: vec![
                EpochOption {
                    epoch: 0,
                    value: None
                };
                node_count
            ],
            scratch: vec![
                EpochOption {
                    epoch: 0,
                    value: None
                };
                node_count
            ],
        }
    }

    fn reset(&mut self, max_pull: usize, upper_bound: Bound<C>, node_count: usize) {
        self.max_pull = max_pull.max(Self::MIN_PULL);
        self.max_block = self
            .max_pull
            .saturating_mul(2)
            .min(self.max_pull.saturating_add(Self::EXTRA_BLOCK));
        self.upper_bound = upper_bound;
        self.len = 0;
        self.blocks.clear();
        self.d0.clear();
        self.d1.clear();
        if self.best.len() < node_count {
            self.best.resize(
                node_count,
                EpochOption {
                    epoch: 0,
                    value: None,
                },
            );
        } else if self.best.len() > node_count {
            self.best.truncate(node_count);
        }
        if self.loc.len() < node_count {
            self.loc.resize(
                node_count,
                EpochOption {
                    epoch: 0,
                    value: None,
                },
            );
        } else if self.loc.len() > node_count {
            self.loc.truncate(node_count);
        }
        if self.scratch.len() < node_count {
            self.scratch.resize(
                node_count,
                EpochOption {
                    epoch: 0,
                    value: None,
                },
            );
        } else if self.scratch.len() > node_count {
            self.scratch.truncate(node_count);
        }
        self.bump_epoch();
        self.bump_scratch_epoch();
    }

    const fn is_empty(&self) -> bool {
        self.len == 0
    }

    fn bump_epoch(&mut self) {
        self.epoch = self.epoch.wrapping_add(1);
        if self.epoch == 0 {
            self.epoch = 1;
            for entry in &mut self.best {
                entry.clear();
            }
            for entry in &mut self.loc {
                entry.clear();
            }
        }
    }

    fn bump_scratch_epoch(&mut self) {
        self.scratch_epoch = self.scratch_epoch.wrapping_add(1);
        if self.scratch_epoch == 0 {
            self.scratch_epoch = 1;
            for entry in &mut self.scratch {
                entry.clear();
            }
        }
    }

    fn best_get(&self, key: usize) -> Option<Label<C>> {
        self.best[key].get(self.epoch)
    }

    fn best_value(&self, key: usize) -> Label<C> {
        self.best[key].get(self.epoch).expect("best label missing")
    }

    fn best_set(&mut self, key: usize, label: Label<C>) {
        self.best[key].set(self.epoch, label);
    }

    fn best_clear(&mut self, key: usize) {
        self.best[key].clear();
    }

    fn loc_take(&mut self, key: usize) -> Option<Location> {
        let loc = self.loc[key].get(self.epoch);
        if loc.is_some() {
            self.loc[key].clear();
        }
        loc
    }

    fn scratch_get(&self, key: usize) -> Option<Label<C>> {
        self.scratch[key].get(self.scratch_epoch)
    }

    fn scratch_set(&mut self, key: usize, label: Label<C>) {
        self.scratch[key].set(self.scratch_epoch, label);
    }

    fn scratch_take(&mut self, key: usize) -> Option<Label<C>> {
        let label = self.scratch[key].get(self.scratch_epoch);
        if label.is_some() {
            self.scratch[key].clear();
        }
        label
    }

    fn insert(&mut self, key: usize, label: Label<C>) {
        if !self.prepare_insert(key, label) {
            return;
        }

        let Some(block_id) = self.find_d1_block(label) else {
            self.create_block(BlockKind::D1, vec![key], false);
            return;
        };

        self.insert_into_block(block_id, key);
        if self.blocks[block_id]
            .as_ref()
            .is_some_and(|block| block.items.len() > self.max_block)
        {
            self.split_block(block_id);
        }
    }

    fn batch_prepend(&mut self, items: impl IntoIterator<Item = (usize, Label<C>)>) {
        self.bump_scratch_epoch();
        let mut keys = Vec::new();
        for (key, label) in items {
            if let Some(cur) = self.best_get(key) {
                if label >= cur {
                    continue;
                }
            }
            match self.scratch_get(key) {
                None => {
                    self.scratch_set(key, label);
                    keys.push(key);
                }
                Some(existing) => {
                    if label < existing {
                        self.scratch_set(key, label);
                    }
                }
            }
        }

        if keys.is_empty() {
            return;
        }

        let mut batch: Vec<(usize, Label<C>)> = Vec::with_capacity(keys.len());
        for key in keys {
            let label = self.scratch_take(key).unwrap();
            if self.best_get(key).is_some() {
                self.remove_key(key);
            }
            self.best_set(key, label);
            batch.push((key, label));
        }

        if batch.len() <= self.max_block {
            let items = batch.into_iter().map(|(key, _)| key).collect::<Vec<_>>();
            self.create_block(BlockKind::D0, items, true);
            return;
        }

        batch.sort_unstable_by(|a, b| a.1.cmp(&b.1));
        for chunk in batch.chunks(self.max_block).rev() {
            let items = chunk.iter().map(|(key, _)| *key).collect::<Vec<_>>();
            self.create_block(BlockKind::D0, items, true);
        }
    }

    fn pull(&mut self) -> (Bound<C>, Vec<usize>) {
        if self.len == 0 {
            return (self.upper_bound, Vec::new());
        }

        let mut candidate_blocks = Vec::new();
        let mut candidate_count = 0usize;

        while self.d0.front().is_some_and(|&id| self.blocks[id].is_none()) {
            self.d0.pop_front();
        }

        for &block_id in &self.d0 {
            let Some(block) = self.blocks[block_id].as_ref() else {
                continue;
            };
            candidate_count = candidate_count.saturating_add(block.items.len());
            candidate_blocks.push(block_id);
            if candidate_count >= self.max_pull {
                break;
            }
        }

        if candidate_count < self.max_pull {
            for &block_id in self.d1.values() {
                let block = self.blocks[block_id].as_ref().unwrap();
                candidate_count = candidate_count.saturating_add(block.items.len());
                candidate_blocks.push(block_id);
                if candidate_count >= self.max_pull {
                    break;
                }
            }
        }

        let mut candidates = Vec::with_capacity(candidate_count);
        for &block_id in &candidate_blocks {
            let block = self.blocks[block_id].as_ref().unwrap();
            candidates.extend(block.items.iter().copied());
        }

        if candidates.len() <= self.max_pull {
            for key in candidates.iter().copied() {
                self.remove_key(key);
            }
            return (self.upper_bound, candidates);
        }

        let mid = self.max_pull;
        candidates
            .select_nth_unstable_by(mid, |a, b| self.best_value(*a).cmp(&self.best_value(*b)));

        let (out, remaining) = candidates.split_at(mid);
        let mut boundary = None;
        for key in remaining {
            let label = self.best_value(*key);
            boundary = match boundary {
                None => Some(label),
                Some(current) => Some(current.min(label)),
            };
        }

        let out = out.to_vec();
        for key in out.iter().copied() {
            self.remove_key(key);
        }

        (boundary.or(self.upper_bound), out)
    }

    fn prepare_insert(&mut self, key: usize, label: Label<C>) -> bool {
        if let Some(cur) = self.best_get(key) {
            if label >= cur {
                return false;
            }
            self.remove_key(key);
        }
        self.best_set(key, label);
        true
    }

    fn find_d1_block(&self, label: Label<C>) -> Option<usize> {
        self.d1.range((label, 0)..).next().map(|(_, &id)| id)
    }

    fn d1_insert(&mut self, max: Label<C>, block_id: usize) {
        self.d1.insert((max, block_id), block_id);
    }

    fn d1_remove(&mut self, max: Label<C>, block_id: usize) {
        self.d1.remove(&(max, block_id));
    }

    fn insert_into_block(&mut self, block_id: usize, key: usize) {
        let label = self.best_value(key);
        let (needs_update, index) = {
            let block = self.blocks[block_id].as_mut().unwrap();
            let index = block.items.len();
            block.items.push(key);
            (label > block.max, index)
        };
        self.loc[key].set(self.epoch, Location { block_id, index });
        self.len = self.len.saturating_add(1);
        if needs_update {
            self.update_block_max(block_id, label, key);
        }
    }

    fn create_block(&mut self, kind: BlockKind, items: Vec<usize>, prepend_d0: bool) -> usize {
        if items.is_empty() {
            return usize::MAX;
        }
        let (max, max_key) = self.block_max(&items);
        let block_id = self.blocks.len();
        for (index, key) in items.iter().copied().enumerate() {
            self.loc[key].set(self.epoch, Location { block_id, index });
        }
        self.len = self.len.saturating_add(items.len());
        self.blocks.push(Some(Block {
            items,
            max,
            max_key,
            kind,
        }));
        match kind {
            BlockKind::D0 => {
                if prepend_d0 {
                    self.d0.push_front(block_id);
                } else {
                    self.d0.push_back(block_id);
                }
            }
            BlockKind::D1 => {
                self.d1_insert(max, block_id);
            }
        }
        block_id
    }

    fn update_block_max(&mut self, block_id: usize, max: Label<C>, max_key: usize) {
        let kind = self.blocks[block_id].as_ref().unwrap().kind;
        if kind == BlockKind::D1 {
            let old_max = self.blocks[block_id].as_ref().unwrap().max;
            self.d1_remove(old_max, block_id);
            let block = self.blocks[block_id].as_mut().unwrap();
            block.max = max;
            block.max_key = max_key;
            self.d1_insert(max, block_id);
        } else {
            let block = self.blocks[block_id].as_mut().unwrap();
            block.max = max;
            block.max_key = max_key;
        }
    }

    fn remove_key(&mut self, key: usize) {
        let Some(loc) = self.loc_take(key) else {
            self.best_clear(key);
            return;
        };
        let block_id = loc.block_id;
        let new_index = loc.index;
        let mut missing = false;
        let mut moved_loc: Option<(usize, usize)> = None;
        let (block_empty, block_kind, block_max, needs_recompute) = {
            let block = self.blocks[block_id].as_mut().unwrap();
            let index = if new_index < block.items.len() && block.items[new_index] == key {
                new_index
            } else if let Some(pos) = block.items.iter().position(|&k| k == key) {
                pos
            } else {
                missing = true;
                0
            };
            if missing {
                (false, block.kind, block.max, false)
            } else {
                let moved = block.items.swap_remove(index);
                if moved != key {
                    moved_loc = Some((moved, index));
                }
                (
                    block.items.is_empty(),
                    block.kind,
                    block.max,
                    block.max_key == key,
                )
            }
        };
        if missing {
            self.best_clear(key);
            return;
        }
        if let Some((moved, index)) = moved_loc {
            self.loc[moved].set(self.epoch, Location { block_id, index });
        }
        self.len = self.len.saturating_sub(1);
        self.best_clear(key);

        if block_empty {
            self.blocks[block_id] = None;
            if block_kind == BlockKind::D1 {
                self.d1_remove(block_max, block_id);
            }
            return;
        }

        if needs_recompute {
            let (max, max_key) = {
                let items = &self.blocks[block_id].as_ref().unwrap().items;
                self.block_max(items)
            };
            self.update_block_max(block_id, max, max_key);
        }
    }

    fn block_max(&self, items: &[usize]) -> (Label<C>, usize) {
        let mut max_key = items[0];
        let mut max_label = self.best_value(max_key);
        for &key in &items[1..] {
            let label = self.best_value(key);
            if label > max_label {
                max_label = label;
                max_key = key;
            }
        }
        (max_label, max_key)
    }

    fn split_block(&mut self, block_id: usize) {
        let (kind, should_split) = {
            let block = self.blocks[block_id].as_ref().unwrap();
            (block.kind, block.items.len() > self.max_block)
        };
        if !should_split || kind != BlockKind::D1 {
            return;
        }

        let mut items = {
            let block = self.blocks[block_id].as_mut().unwrap();
            let mut items = Vec::new();
            items.append(&mut block.items);
            items
        };
        let mid = items.len() / 2;
        items.select_nth_unstable_by(mid, |a, b| self.best_value(*a).cmp(&self.best_value(*b)));
        let right_items = items.split_off(mid);
        let left_items = items;

        let (left_max, left_key) = self.block_max(&left_items);
        for (index, key) in left_items.iter().copied().enumerate() {
            self.loc[key].set(self.epoch, Location { block_id, index });
        }
        {
            let block = self.blocks[block_id].as_mut().unwrap();
            block.items = left_items;
        }
        self.update_block_max(block_id, left_max, left_key);

        let right_len = right_items.len();
        self.create_block(BlockKind::D1, right_items, false);
        self.len = self.len.saturating_sub(right_len);
    }
}

struct PortGraph<C> {
    adjacency: Vec<Vec<(usize, C)>>,
    in_ports_by_node: Vec<Vec<usize>>,
    in_port_source: Vec<Option<usize>>,
    start_port: usize,
    original_edges: Vec<Vec<(usize, C)>>,
}

fn build_port_graph<C, FN, IN>(
    start: usize,
    number_of_nodes: usize,
    successors: &mut FN,
) -> PortGraph<C>
where
    C: Zero + Ord + Copy,
    FN: FnMut(usize) -> IN,
    IN: IntoIterator<Item = (usize, C)>,
{
    let mut out_edges: Vec<Vec<(usize, C)>> = vec![Vec::new(); number_of_nodes];
    let mut in_degree = vec![0usize; number_of_nodes];
    for (u, edges) in out_edges.iter_mut().enumerate() {
        let iter = successors(u).into_iter();
        let (lower, upper) = iter.size_hint();
        edges.reserve(upper.unwrap_or(lower));
        for (v, w) in iter {
            edges.push((v, w));
            if let Some(entry) = in_degree.get_mut(v) {
                *entry = entry.saturating_add(1);
            }
        }
    }

    let mut port_counts = vec![0usize; number_of_nodes];
    for v in 0..number_of_nodes {
        let count = out_edges[v].len().saturating_add(in_degree[v]);
        port_counts[v] = if count == 0 && v == start { 1 } else { count };
    }

    let mut port_base = vec![0usize; number_of_nodes];
    let mut total_ports = 0usize;
    for v in 0..number_of_nodes {
        port_base[v] = total_ports;
        total_ports = total_ports.saturating_add(port_counts[v]);
    }

    let mut adjacency = vec![Vec::new(); total_ports];
    let mut in_ports_by_node = vec![Vec::new(); number_of_nodes];
    let mut in_port_source = vec![None; total_ports];

    for v in 0..number_of_nodes {
        let count = port_counts[v];
        if count == 0 {
            continue;
        }
        let base = port_base[v];
        if count > 1 {
            for i in 0..count {
                let from = base + i;
                let to = base + (i + 1) % count;
                adjacency[from].push((to, C::zero()));
            }
        }
    }

    let mut next_in = vec![0usize; number_of_nodes];
    for u in 0..number_of_nodes {
        let out_base = port_base[u];
        for (out_idx, (v, w)) in out_edges[u].iter().copied().enumerate() {
            let in_base = port_base[v] + out_edges[v].len();
            let in_idx = next_in[v];
            next_in[v] = next_in[v].saturating_add(1);
            let out_port = out_base + out_idx;
            let in_port = in_base + in_idx;
            adjacency[out_port].push((in_port, w));
            in_port_source[in_port] = Some(u);
            in_ports_by_node[v].push(in_port);
        }
    }

    let start_port = port_base[start];
    PortGraph {
        adjacency,
        in_ports_by_node,
        in_port_source,
        start_port,
        original_edges: out_edges,
    }
}

fn dijkstra_all_indexed<C>(start: usize, adjacency: &[Vec<(usize, C)>]) -> Vec<Option<(usize, C)>>
where
    C: Zero + Ord + Copy,
{
    let parents =
        crate::directed::dijkstra::dijkstra_all(&start, |node| adjacency[*node].iter().copied());
    let mut out = vec![None; adjacency.len()];
    for (node, (parent, cost)) in parents {
        out[node] = Some((parent, cost));
    }
    out
}

fn distances_are_relaxed<C>(
    start: usize,
    distances: &[Option<C>],
    adjacency: &[Vec<(usize, C)>],
) -> bool
where
    C: Zero + Ord + Copy,
{
    if distances.len() != adjacency.len() {
        return false;
    }
    if let Some(Some(value)) = distances.get(start) {
        if *value != C::zero() {
            return false;
        }
    }
    for (u, edges) in adjacency.iter().enumerate() {
        let Some(du) = distances[u] else {
            continue;
        };
        for &(v, w) in edges {
            let candidate = du + w;
            match distances.get(v).and_then(|d| *d) {
                Some(dv) => {
                    if candidate < dv {
                        return false;
                    }
                }
                None => {
                    return false;
                }
            }
        }
    }
    true
}

fn find_pivots<C, FN, IN>(
    ctx: &mut IndexedContext<C>,
    bound: Bound<C>,
    sources: &[usize],
    params: &Params,
    successors: &mut FN,
) -> (Vec<usize>, Vec<usize>)
where
    C: Zero + Ord + Copy,
    FN: FnMut(usize) -> IN,
    IN: IntoIterator<Item = (usize, C)>,
{
    let in_w_epoch = ctx.scratch.in_w.next_epoch();
    let mut w: Vec<usize> = Vec::new();
    let mut frontier: Vec<usize> = Vec::new();
    for &s in sources {
        if !ctx.scratch.in_w.is_marked(s, in_w_epoch) {
            ctx.scratch.in_w.mark(s, in_w_epoch);
            w.push(s);
            frontier.push(s);
        }
    }

    // k-step Bellman-Ford-like relaxations restricted by `bound` only for expanding `W`.
    for _ in 0..params.k {
        let mut next = Vec::new();
        for &u_idx in &frontier {
            for (v_idx, weight) in successors(u_idx) {
                let candidate = label_add(ctx, u_idx, v_idx, weight);
                if !bound_lt(candidate, bound) || !try_relax(ctx, v_idx, candidate) {
                    continue;
                }
                if !ctx.scratch.in_w.is_marked(v_idx, in_w_epoch) {
                    ctx.scratch.in_w.mark(v_idx, in_w_epoch);
                    w.push(v_idx);
                    next.push(v_idx);
                }
            }
        }
        frontier = next;
        if w.len() > params.k.saturating_mul(sources.len().max(1)) {
            // Early exit: keep all sources as pivots.
            return (sources.to_vec(), w);
        }
        if frontier.is_empty() {
            break;
        }
    }

    // Build the forest induced by predecessor pointers on `W`.
    {
        let scratch = &mut ctx.scratch;
        for &v in &w {
            scratch.children[v].clear();
            scratch.parent_in_w[v] = usize::MAX;
            scratch.subtree_size[v] = 0;
        }
    }
    for &v in &w {
        let p = ctx.state(v).pred;
        if p != usize::MAX && ctx.scratch.in_w.is_marked(p, in_w_epoch) {
            ctx.scratch.children[p].push(v);
            ctx.scratch.parent_in_w[v] = p;
        }
    }

    // Compute subtree sizes for all roots (postorder).
    let scratch = &mut ctx.scratch;
    for &root in &w {
        if scratch.parent_in_w[root] != usize::MAX {
            continue;
        }
        let mut stack: Vec<(usize, usize)> = vec![(root, 0)];
        while let Some((node, next_child)) = stack.pop() {
            if next_child < scratch.children[node].len() {
                stack.push((node, next_child + 1));
                stack.push((scratch.children[node][next_child], 0));
            } else {
                let mut size = 1usize;
                for &c in &scratch.children[node] {
                    size = size.saturating_add(scratch.subtree_size[c]);
                }
                scratch.subtree_size[node] = size;
            }
        }
    }

    let mut pivots = Vec::new();
    for &u in sources {
        if scratch.parent_in_w[u] == usize::MAX && scratch.subtree_size[u] >= params.k {
            pivots.push(u);
        }
    }

    (pivots, w)
}

fn base_case<C, FN, IN>(
    ctx: &mut IndexedContext<C>,
    bound: Bound<C>,
    sources: &[usize],
    params: &Params,
    successors: &mut FN,
) -> (Bound<C>, Vec<usize>)
where
    C: Zero + Ord + Copy,
    FN: FnMut(usize) -> IN,
    IN: IntoIterator<Item = (usize, C)>,
{
    let Some(&x) = sources.first() else {
        return (bound, Vec::new());
    };
    debug_assert!(ctx.state(x).label.is_some());

    let mut heap: BinaryHeap<Reverse<(Label<C>, usize)>> = BinaryHeap::new();
    heap.push(Reverse((ctx.state(x).label.unwrap(), x)));

    let extracted_epoch = ctx.scratch.extracted.next_epoch();
    let mut u0: Vec<usize> = Vec::new();

    while let Some(Reverse((label_u, u_idx))) = heap.pop() {
        if ctx.state(u_idx).label.is_none_or(|l| l != label_u) {
            continue; // stale
        }
        if ctx.scratch.extracted.is_marked(u_idx, extracted_epoch) {
            continue;
        }
        ctx.scratch.extracted.mark(u_idx, extracted_epoch);
        u0.push(u_idx);
        if u0.len() > params.k {
            break;
        }
        for (v_idx, w) in successors(u_idx) {
            let candidate = label_add(ctx, u_idx, v_idx, w);
            if bound_lt(candidate, bound) && try_relax(ctx, v_idx, candidate) {
                heap.push(Reverse((candidate, v_idx)));
            }
        }
    }

    // Mark extracted nodes as complete (including the boundary vertex, if any).
    for &u in &u0 {
        ctx.state_mut(u).complete = true;
    }

    if u0.len() <= params.k {
        return (bound, u0);
    }

    let b_prime = u0
        .iter()
        .filter_map(|&u| ctx.state(u).label)
        .max()
        .map_or(bound, Some);
    let u = u0
        .into_iter()
        .filter(|&u| ctx.state(u).label.is_some_and(|l| bound_lt(l, b_prime)))
        .collect::<Vec<_>>();
    (b_prime, u)
}

fn bmssp_rec<C, FN, IN>(
    ctx: &mut IndexedContext<C>,
    layer: usize,
    bound: Bound<C>,
    sources: &[usize],
    params: &Params,
    successors: &mut FN,
) -> (Bound<C>, Vec<usize>)
where
    C: Zero + Ord + Copy,
    FN: FnMut(usize) -> IN,
    IN: IntoIterator<Item = (usize, C)>,
{
    if layer == 0 {
        return base_case(ctx, bound, sources, params, successors);
    }

    let (pivots, w) = find_pivots(ctx, bound, sources, params, successors);

    let m = pow2_sat((layer - 1).saturating_mul(params.t));
    let mut ds = ctx.take_queue(m.max(1), bound);
    for &p in &pivots {
        if let Some(label) = ctx.state(p).label {
            ds.insert(p, label);
        }
    }

    let budget = params
        .k
        .saturating_mul(pow2_sat(layer.saturating_mul(params.t)));
    let out_epoch = ctx.scratch.out_set.next_epoch();
    let mut out: Vec<usize> = Vec::new();

    let mut current_b = pivots
        .iter()
        .filter_map(|&p| ctx.state(p).label)
        .min()
        .map_or(bound, Some);

    while out.len() < budget && !ds.is_empty() {
        let (b_i, s_i) = ds.pull();
        let (b_prime_i, u_i) = bmssp_rec(ctx, layer - 1, b_i, &s_i, params, successors);
        current_b = b_prime_i;

        for &u in &u_i {
            ctx.state_mut(u).complete = true;
            if !ctx.scratch.out_set.is_marked(u, out_epoch) {
                ctx.scratch.out_set.mark(u, out_epoch);
                out.push(u);
            }
        }

        let mut k_list: Vec<(usize, Label<C>)> = Vec::new();
        for &u_idx in &u_i {
            for (v_idx, w_uv) in successors(u_idx) {
                let candidate = label_add(ctx, u_idx, v_idx, w_uv);
                if !try_relax(ctx, v_idx, candidate) {
                    continue;
                }
                if !bound_lt(candidate, bound) {
                    continue;
                }
                if in_range(candidate, b_i, bound) {
                    ds.insert(v_idx, candidate);
                } else if in_range(candidate, b_prime_i, b_i) {
                    k_list.push((v_idx, candidate));
                }
            }
        }

        // Re-add sources from this pull if the recursive call was partial.
        for &x in &s_i {
            if let Some(label) = ctx.state(x).label {
                if in_range(label, b_prime_i, b_i) {
                    k_list.push((x, label));
                }
            }
        }

        ds.batch_prepend(k_list);
    }

    let b_return = if ds.is_empty() {
        bound
    } else {
        bound_min(current_b, bound)
    };

    // Add complete vertices from `W` that are below the returned boundary.
    for x in w {
        if let Some(label) = ctx.state(x).label {
            if bound_lt(label, b_return) {
                ctx.state_mut(x).complete = true;
                if !ctx.scratch.out_set.is_marked(x, out_epoch) {
                    ctx.scratch.out_set.mark(x, out_epoch);
                    out.push(x);
                }
            }
        }
    }

    let result = (b_return, out);
    ctx.recycle_queue(ds);
    result
}

fn build_path_indexed<C>(target: usize, parents: &[Option<(usize, C)>]) -> Vec<usize> {
    let mut rev = vec![target];
    let mut next = target;
    while let Some(Some((parent, _))) = parents.get(next) {
        rev.push(*parent);
        next = *parent;
    }
    rev.reverse();
    rev
}

/// Compute all reachable nodes from a starting point as well as the
/// minimum cost to reach them and a possible optimal parent node,
/// using the BMSSP-based SSSP algorithm on dense `usize` indices.
///
/// - `start` is the starting node index.
/// - `successors` returns a list of successors for a given node index, along with the cost for
///   moving from the node to the successor. This cost must be non-negative.
/// - `number_of_nodes` is the number of vertices in the graph (or an upper bound on the number of
///   reachable vertices). Successor indices must be in the range `[0, number_of_nodes)`.
/// - This implementation applies a constant-degree port-graph transformation internally, which
///   adds preprocessing time and memory overhead.
/// - If any projected distance can still be improved by an edge relaxation, the implementation
///   falls back to indexed Dijkstra for correctness.
///
/// The result is a vector of length `number_of_nodes` where every reachable node (not including
/// `start`) is associated with an optimal parent node and a cost from the start node.
///
/// # Example
///
/// ```
/// use pathfinding_faster::IndexedGraph;
///
/// let graph = IndexedGraph::from_adjacency(vec![
///     vec![(1, 10), (2, 10)],
///     vec![(3, 10), (4, 10)],
///     vec![(5, 10), (6, 10)],
///     vec![(7, 10), (8, 10)],
///     vec![],
///     vec![],
///     vec![],
///     vec![],
///     vec![],
/// ]);
/// let parents = graph.bmssp_all(0);
/// assert_eq!(parents[1], Some((0, 10)));
/// assert_eq!(parents[8], Some((3, 30)));
/// ```
pub fn bmssp_all_indexed<C, FN, IN>(
    start: usize,
    mut successors: FN,
    number_of_nodes: usize,
) -> Vec<Option<(usize, C)>>
where
    C: Zero + Ord + Copy,
    FN: FnMut(usize) -> IN,
    IN: IntoIterator<Item = (usize, C)>,
{
    if number_of_nodes == 0 {
        return Vec::new();
    }
    debug_assert!(start < number_of_nodes);

    let port_graph = build_port_graph(start, number_of_nodes, &mut successors);
    if port_graph.adjacency.is_empty() {
        return vec![None; number_of_nodes];
    }

    let port_count = port_graph.adjacency.len();
    let logn = log2_ceil(number_of_nodes.max(2));
    let k = cube_root_floor(logn).max(1);
    let t = cube_root_floor(logn.saturating_mul(logn)).max(1);
    let l = logn.div_ceil(t);
    let params = Params { k, t, l };

    let start_label = Label {
        cost: C::zero(),
        hops: 0,
        node: port_graph.start_port,
        pred: usize::MAX,
    };
    let mut ctx = IndexedContext::new(port_count, port_graph.start_port, start_label);

    let _ = bmssp_rec(
        &mut ctx,
        params.l,
        None,
        &[port_graph.start_port],
        &params,
        &mut |node| port_graph.adjacency[node].iter().copied(),
    );

    let mut out = vec![None; number_of_nodes];
    for (idx, entry) in out.iter_mut().enumerate() {
        if idx == start {
            continue;
        }
        let mut best: Option<(Label<C>, usize)> = None;
        for &port in &port_graph.in_ports_by_node[idx] {
            let Some(label) = ctx.state(port).label else {
                continue;
            };
            best = match best {
                None => Some((label, port)),
                Some((best_label, _)) if label < best_label => Some((label, port)),
                Some(current) => Some(current),
            };
        }
        let Some((label, port)) = best else {
            continue;
        };
        let Some(pred) = port_graph.in_port_source[port] else {
            continue;
        };
        *entry = Some((pred, label.cost));
    }
    let mut distances = vec![None; number_of_nodes];
    distances[start] = Some(C::zero());
    for (idx, entry) in out.iter().enumerate() {
        if let Some((_, cost)) = entry {
            distances[idx] = Some(*cost);
        }
    }
    if !distances_are_relaxed(start, &distances, &port_graph.original_edges) {
        return dijkstra_all_indexed(start, &port_graph.original_edges);
    }
    out
}

/// Compute a shortest path using the BMSSP-based SSSP algorithm on dense `usize` indices.
///
/// This is a convenience wrapper around [`bmssp_all_indexed`]: it computes all reachable nodes
/// first, then returns the shortest path to the reachable node satisfying `success` with minimal
/// cost.
///
/// - `start` is the starting node index.
/// - `successors` returns a list of successors for a given node index, along with the cost for
///   moving from the node to the successor. This cost must be non-negative.
/// - `success` checks whether the goal has been reached.
/// - `number_of_nodes` is the number of vertices in the graph (or an upper bound on the number of
///   reachable vertices). Successor indices must be in the range `[0, number_of_nodes)`.
/// - This implementation applies a constant-degree port-graph transformation internally, which
///   adds preprocessing time and memory overhead.
/// - If any projected distance can still be improved by an edge relaxation, the implementation
///   falls back to indexed Dijkstra for correctness.
///
/// # Example
///
/// ```
/// use pathfinding_faster::IndexedGraph;
///
/// let graph = IndexedGraph::from_adjacency(vec![
///     vec![(1, 10), (2, 10)],
///     vec![(3, 10), (4, 10)],
///     vec![(5, 10), (6, 10)],
///     vec![(7, 10), (8, 10)],
///     vec![],
///     vec![],
///     vec![],
///     vec![],
///     vec![],
/// ]);
/// let result = graph.bmssp(0, |n| n == 8);
/// assert_eq!(result, Some((vec![0, 1, 3, 8], 30)));
/// ```
pub fn bmssp_indexed<C, FN, IN, FS>(
    start: usize,
    successors: FN,
    mut success: FS,
    number_of_nodes: usize,
) -> Option<(Vec<usize>, C)>
where
    C: Zero + Ord + Copy,
    FN: FnMut(usize) -> IN,
    IN: IntoIterator<Item = (usize, C)>,
    FS: FnMut(usize) -> bool,
{
    if number_of_nodes == 0 {
        return None;
    }
    if success(start) {
        return Some((vec![start], C::zero()));
    }

    let parents = bmssp_all_indexed(start, successors, number_of_nodes);
    let mut best: Option<(usize, C)> = None;
    for (node, entry) in parents.iter().enumerate() {
        let Some((_, cost)) = entry else { continue };
        if success(node) {
            best = match best {
                None => Some((node, *cost)),
                Some((_, best_cost)) if *cost < best_cost => Some((node, *cost)),
                Some(x) => Some(x),
            };
        }
    }

    best.map(|(target, cost)| (build_path_indexed(target, &parents), cost))
}

#[cfg(test)]
mod tests {
    use crate::IndexedGraph;

    #[test]
    fn bmssp_matches_dijkstra_on_multi_degree_graph() {
        let graph = IndexedGraph::from_adjacency(vec![
            vec![(1, 2), (2, 1), (3, 10)],
            vec![(3, 1)],
            vec![(1, 1), (3, 5)],
            vec![(4, 1)],
            vec![],
            vec![],
        ]);
        let bmssp = graph.bmssp_all(0);
        let dijkstra = graph.dijkstra_all(0);
        assert_eq!(bmssp.len(), dijkstra.len());
        for idx in 0..bmssp.len() {
            let bm_cost = bmssp[idx].map(|(_, cost)| cost);
            let dj_cost = dijkstra[idx].map(|(_, cost)| cost);
            assert_eq!(bm_cost, dj_cost, "node {idx}");
        }
    }
}
