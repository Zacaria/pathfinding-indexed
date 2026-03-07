## ADDED Requirements
### Requirement: Indexed-only public API surface
The system SHALL expose the indexed graph types and their algorithm methods as the primary
public API. The crate SHALL NOT export `grid`, `matrix`, `utils`, `kuhn_munkres`,
`noderefs`, or cycle detection modules.

#### Scenario: Public modules
- **WHEN** a user inspects the public modules of the crate
- **THEN** only the indexed graph types and their algorithm methods are available
- **AND** the excluded modules are not exported

### Requirement: Crate identity
The system SHALL identify as `pathfinding-faster` in package metadata and documentation.

#### Scenario: Package metadata
- **WHEN** a user reads the crate metadata and README
- **THEN** the crate name is `pathfinding-faster` and examples use that name
