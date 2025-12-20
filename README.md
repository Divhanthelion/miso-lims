# MISO LIMS - Rust Rewrite

A high-performance Laboratory Information Management System (LIMS) for Next-Generation Sequencing (NGS) centers, rewritten in Rust.

## Overview

MISO (Managing Information for Sequencing Operations) is an open-source LIMS specifically architected for the unique challenges of NGS facilities. This Rust implementation provides:

- **Memory Safety**: No garbage collection pauses during critical operations
- **High Performance**: Native binaries with minimal resource consumption
- **Type Safety**: Compile-time validation of biological workflow constraints
- **Async I/O**: Efficient handling of hardware integrations and concurrent users

## Architecture

```
miso-lims/
├── crates/
│   ├── miso-domain/        # Core domain entities and business logic
│   ├── miso-application/   # Application services and use cases
│   ├── miso-infrastructure/# Database and hardware implementations
│   ├── miso-api/           # Axum REST API server
│   ├── miso-migration/     # Database migrations
│   └── miso-frontend/      # Leptos WASM frontend (WIP)
├── docker-compose.yml      # Docker deployment
└── Dockerfile              # Multi-stage build
```

### Technology Stack

| Component | Technology | Purpose |
|-----------|------------|---------|
| Web Framework | Axum | High-performance async HTTP |
| Database ORM | SeaORM | Async MySQL access |
| Async Runtime | Tokio | Non-blocking I/O |
| Frontend | Leptos | Reactive WASM UI |
| Authentication | JWT + LDAP | Secure access control |

## Features

### Sample Management
- **Plain Mode**: Simple Sample → Library → Pool workflow
- **Detailed Mode**: Full hierarchy (Identity → Tissue → Stock → Aliquot)
- Barcode tracking and validation
- QC status management

### Library Preparation
- DNA index management with collision detection
- Hamming distance validation for pooling
- Kit and protocol tracking

### Hardware Integration
- VisionMate 2D barcode scanners (async TCP)
- Zebra label printers (ZPL)
- Sequencer monitoring (Run Scanner)

### Storage Management
- Freezer → Shelf → Rack → Box hierarchy
- 96-well and 384-well plate support
- Visual plate map interface

## Getting Started

### Prerequisites

- Rust 1.83+ (stable)
- Docker and Docker Compose
- MySQL 8.0+

### Quick Start with Docker

1. Create secrets directory:
```bash
mkdir -p secrets
echo "rootpassword" > secrets/db_root_password.txt
echo "miso" > secrets/db_password.txt
echo "your-jwt-secret-key" > secrets/jwt_secret.txt
```

2. Start the services:
```bash
docker-compose up -d
```

3. Access the API:
```bash
curl http://localhost:8080/health
```

### Development Setup

1. Clone and build:
```bash
git clone https://github.com/miso-lims/miso-lims-rust
cd miso-lims-rust
cargo build
```

2. Set up environment:
```bash
cp .env.example .env
# Edit .env with your database credentials
```

3. Run migrations:
```bash
cargo run --bin miso-migrate -- up
```

4. Start the server:
```bash
cargo run --bin miso-server
```

## API Documentation

### Health Endpoints

```
GET /health     - Liveness check
GET /ready      - Readiness check (DB connectivity)
```

### Projects

```
GET    /api/v1/projects          - List all projects
POST   /api/v1/projects          - Create a project
GET    /api/v1/projects/:id      - Get project details
PUT    /api/v1/projects/:id      - Update a project
DELETE /api/v1/projects/:id      - Delete a project
```

### Samples

```
GET    /api/v1/samples                    - List samples
POST   /api/v1/samples                    - Create a sample
GET    /api/v1/samples/:id                - Get sample details
PUT    /api/v1/samples/:id                - Update a sample
DELETE /api/v1/samples/:id                - Delete a sample
GET    /api/v1/samples/barcode/:barcode   - Find by barcode
GET    /api/v1/samples/project/:id        - List by project
```

### Scanner

```
GET  /api/v1/scanner/status  - Check scanner connectivity
POST /api/v1/scanner/scan    - Trigger rack scan
```

## Configuration

Environment variables:

| Variable | Default | Description |
|----------|---------|-------------|
| `DATABASE_URL` | - | MySQL connection string |
| `JWT_SECRET` | - | Secret for JWT signing |
| `HOST` | 0.0.0.0 | Server bind address |
| `PORT` | 8080 | Server port |
| `LOG_LEVEL` | info | Logging verbosity |
| `CORS_ENABLED` | false | Enable CORS headers |

## Migration from Java MISO

This Rust implementation is designed to run alongside the legacy Java MISO using the Strangler Fig pattern:

1. **Phase 1**: Deploy as read-only (Pinery API replacement)
2. **Phase 2**: Migrate hardware integrations (scanner, printer)
3. **Phase 3**: Shadow testing for write operations
4. **Phase 4**: Gradual cutover of endpoints

The Rust application connects to the existing MySQL database, ensuring data compatibility during migration.

## Contributing

Contributions are welcome! Please read our contributing guidelines before submitting PRs.

## License

This project is licensed under the GNU General Public License v3.0 (GPL-3.0), maintaining compatibility with the original MISO LIMS license.

## Acknowledgments

- Ontario Institute for Cancer Research (OICR)
- Earlham Institute
- Original MISO LIMS contributors

