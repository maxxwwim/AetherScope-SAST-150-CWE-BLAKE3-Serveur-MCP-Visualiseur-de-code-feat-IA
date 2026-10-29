use core_graph::manifest::{GalaxyManifest, GuardrailSettings, ProjectManifest, StackType, TopologyMode};
use std::collections::HashMap;

pub struct ArchScaffolder;

impl ArchScaffolder {
    pub fn create_manifest(project_id: &str, stack: StackType, topology: TopologyMode) -> ProjectManifest {
        let mut galaxies = HashMap::new();

        match stack {
            StackType::PureRust => {
                galaxies.insert(
                    "contracts".to_string(),
                    GalaxyManifest {
                        path: "./contracts".to_string(),
                        role: "Common DTOs, wire types and API schemas".to_string(),
                        remote_sync: false,
                        remote_endpoint: None,
                        readme_blake3: None,
                    },
                );
                galaxies.insert(
                    "client".to_string(),
                    GalaxyManifest {
                        path: "./client".to_string(),
                        role: "Desktop/Web frontend view and state management".to_string(),
                        remote_sync: false,
                        remote_endpoint: None,
                        readme_blake3: None,
                    },
                );
                galaxies.insert(
                    "server".to_string(),
                    GalaxyManifest {
                        path: "./server".to_string(),
                        role: "Axum backend, domain logic and API routes".to_string(),
                        remote_sync: false,
                        remote_endpoint: None,
                        readme_blake3: None,
                    },
                );
                galaxies.insert(
                    "data-nodes".to_string(),
                    GalaxyManifest {
                        path: "./data-nodes".to_string(),
                        role: "SQLx / SeaORM migrations and database repositories".to_string(),
                        remote_sync: false,
                        remote_endpoint: None,
                        readme_blake3: None,
                    },
                );
            }
            StackType::PurePythonUv => {
                galaxies.insert(
                    "contracts".to_string(),
                    GalaxyManifest {
                        path: "./contracts".to_string(),
                        role: "Pydantic v2 schemas and shared domain interfaces".to_string(),
                        remote_sync: false,
                        remote_endpoint: None,
                        readme_blake3: None,
                    },
                );
                galaxies.insert(
                    "client".to_string(),
                    GalaxyManifest {
                        path: "./client".to_string(),
                        role: "Streamlit / Reflex or frontend SPA".to_string(),
                        remote_sync: false,
                        remote_endpoint: None,
                        readme_blake3: None,
                    },
                );
                galaxies.insert(
                    "server".to_string(),
                    GalaxyManifest {
                        path: "./server".to_string(),
                        role: "FastAPI REST API, background workers and domain services".to_string(),
                        remote_sync: false,
                        remote_endpoint: None,
                        readme_blake3: None,
                    },
                );
                galaxies.insert(
                    "data-nodes".to_string(),
                    GalaxyManifest {
                        path: "./data-nodes".to_string(),
                        role: "SQLAlchemy 2 models, Alembic migrations and Redis cache".to_string(),
                        remote_sync: false,
                        remote_endpoint: None,
                        readme_blake3: None,
                    },
                );
            }
            StackType::PureNodePnpm => {
                galaxies.insert(
                    "contracts".to_string(),
                    GalaxyManifest {
                        path: "./contracts".to_string(),
                        role: "Zod schemas, tRPC routers, API request/response types".to_string(),
                        remote_sync: false,
                        remote_endpoint: None,
                        readme_blake3: None,
                    },
                );
                galaxies.insert(
                    "client".to_string(),
                    GalaxyManifest {
                        path: "./client".to_string(),
                        role: "Next.js / Vite UI, components and client forms".to_string(),
                        remote_sync: false,
                        remote_endpoint: None,
                        readme_blake3: None,
                    },
                );
                galaxies.insert(
                    "server".to_string(),
                    GalaxyManifest {
                        path: "./server".to_string(),
                        role: "Hono / Fastify server, domain use-cases and WebSocket handlers".to_string(),
                        remote_sync: false,
                        remote_endpoint: None,
                        readme_blake3: None,
                    },
                );
                galaxies.insert(
                    "data-nodes".to_string(),
                    GalaxyManifest {
                        path: "./data-nodes".to_string(),
                        role: "Drizzle / Prisma schema, migrations and repositories".to_string(),
                        remote_sync: false,
                        remote_endpoint: None,
                        readme_blake3: None,
                    },
                );
            }
            StackType::HybridMmoWeb3 => {
                galaxies.insert(
                    "contracts".to_string(),
                    GalaxyManifest {
                        path: "./contracts".to_string(),
                        role: "Smart contracts ABI, WebSocket packet definitions, shared types".to_string(),
                        remote_sync: false,
                        remote_endpoint: None,
                        readme_blake3: None,
                    },
                );
                galaxies.insert(
                    "client".to_string(),
                    GalaxyManifest {
                        path: "./client".to_string(),
                        role: "PixiJS / Svelte WebGL presentation, UI forms and local states".to_string(),
                        remote_sync: false,
                        remote_endpoint: None,
                        readme_blake3: None,
                    },
                );
                galaxies.insert(
                    "server".to_string(),
                    GalaxyManifest {
                        path: "./server".to_string(),
                        role: "Rust WebSocket high-concurrency game loop and domain services".to_string(),
                        remote_sync: topology == TopologyMode::Dissociated,
                        remote_endpoint: if topology == TopologyMode::Dissociated {
                            Some("wss://game-server.internal/aether-sync".to_string())
                        } else {
                            None
                        },
                        readme_blake3: None,
                    },
                );
                galaxies.insert(
                    "data-nodes".to_string(),
                    GalaxyManifest {
                        path: "./data-nodes".to_string(),
                        role: "PostgreSQL player state, Redis pub/sub and Web3 indexers".to_string(),
                        remote_sync: false,
                        remote_endpoint: None,
                        readme_blake3: None,
                    },
                );
            }
        }

        ProjectManifest {
            version: 2,
            project_id: project_id.to_string(),
            stack_type: stack,
            topology_mode: topology,
            galaxies,
            guardrails: GuardrailSettings::default(),
        }
    }
}
