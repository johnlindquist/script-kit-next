# AFK Audit — Surface Coverage Overview

Top-level map of Script Kit GPUI surfaces and the audit stories that prove their behavior. Each node names a story; status suffix reflects the newest pass outcome.


## Map

```mermaid
flowchart TB

  subgraph Main["Main Launcher (NSPanel)"]
    direction TB
  end

  subgraph Subviews["Main-hosted Subviews"]
    direction TB
  end

  subgraph ACP["ACP Chat"]
    direction TB
  end

  subgraph Popups["Attached Popups"]
    direction TB
  end

  subgraph Concurrency["Concurrency & Lifecycle"]
    direction TB
  end

  subgraph Tools["Agentic-Testing Tools"]
    direction TB
  end

  Main --> Subviews
  Main --> Popups
  Subviews --> Popups
  ACP --> Popups
  ACP --> Concurrency
  Main --> Concurrency
  Tools -.->|"extend"| Main
  Tools -.->|"extend"| Subviews
  Tools -.->|"extend"| ACP
  Tools -.->|"extend"| Popups
```

## Coverage stats (Run 2 through Pass #36)


## Edges worth noting
