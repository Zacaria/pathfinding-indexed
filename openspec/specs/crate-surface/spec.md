# crate-surface Specification

## Purpose
Define the indexed-only public API and package identity exposed to crate users.
## Requirements
### Requirement: Indexed-only public API surface
The system SHALL expose the indexed graph types and their algorithm methods as the primary
public API. The crate SHALL NOT export `grid`, `matrix`, `utils`, `kuhn_munkres`,
`noderefs`, or cycle detection modules.

#### Scenario: Public modules
- **WHEN** a user inspects the public modules of the crate
- **THEN** only the indexed graph types and their algorithm methods are available
- **AND** the excluded modules are not exported

### Requirement: Crate identity
The system SHALL identify as `pathfinding-indexed` in package metadata and documentation.

#### Scenario: Package metadata
- **WHEN** a user reads the crate metadata and README
- **THEN** the crate name is `pathfinding-indexed` and examples use that name

### Requirement: Indexed helpers remain adapters into the indexed API
The system SHALL expose grid- and matrix-shaped input helpers as constructors on indexed graph
types or mapping helpers, not as a restoration of the old generic helper modules.

#### Scenario: Public helper surface
- **WHEN** a user reads the crate docs
- **THEN** the helper entry points are documented as ways to build `IndexedGraph` or
  `IndexedGraphMap`
- **AND** the indexed graph types remain the primary API surface
