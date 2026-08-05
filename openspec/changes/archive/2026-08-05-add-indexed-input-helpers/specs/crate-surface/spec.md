## ADDED Requirements

### Requirement: Indexed helpers remain adapters into the indexed API
The system SHALL expose grid- and matrix-shaped input helpers as constructors on indexed graph
types or mapping helpers, not as a restoration of the old generic helper modules.

#### Scenario: Public helper surface
- **WHEN** a user reads the crate docs
- **THEN** the helper entry points are documented as ways to build `IndexedGraph` or
  `IndexedGraphMap`
- **AND** the indexed graph types remain the primary API surface
