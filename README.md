# ☁️ cloudforge

> A unified infrastructure automation CLI that eliminates fragmented cloud toolchains.

---

## 🧩 The Problem

62% of enterprises struggle with fragmented tools and lack of automation, delaying critical
projects. Teams juggle Terraform, Ansible, shell scripts, and vendor CLIs with no coherent
glue — slowing down deployments and burning engineering hours.

---

## 💡 The Solution

`cloudforge` provides a single declarative pipeline engine that orchestrates multi-cloud
provisioning steps in sequence or in parallel — defined in a simple YAML file and executed
via a single binary with zero runtime dependencies.

---

## 🚀 Features

- 📄 **Declarative YAML pipelines** — define your entire workflow in one file
- ⚡ **Parallel & sequential execution** — powered by Tokio async runtime
- 🌍 **Multi-cloud agnostic** — wraps any CLI tool (AWS CLI, gcloud, kubectl, etc.)
- 📦 **Single binary** — no runtime, no dependencies, drop it into any CI/CD agent
- 🔌 **Extensible step system** — plug in custom runners via trait objects

---

## 📦 Installation

```bash
# Build from source
git clone https://github.com/your-org/cloudforge
cd cloudforge
cargo build --release
cp target/release/cloudforge /usr/local/bin/
```

---

## 🛠️ Usage

Define a pipeline in YAML:

```yaml
# pipeline.yaml
name: production-deploy
steps:
  - name: Lint Terraform
    run: terraform fmt -check
  - name: Validate Terraform
    run: terraform validate
  - name: Plan Infrastructure
    run: terraform plan -out=tfplan
    env:
      AWS_REGION: us-east-1
  - name: Apply Infrastructure
    run: terraform apply tfplan
```

Run the pipeline:

```bash
cloudforge run pipeline.yaml
```

---

## 🏗️ Project Structure

```
cloudforge/
├── Cargo.toml
├── README.md
├── src/
│   ├── main.rs          # CLI entrypoint & argument parsing
│   ├── pipeline.rs      # Pipeline & Step deserialization
│   ├── executor.rs      # Step execution engine (sequential + parallel)
│   └── runner/
│       ├── mod.rs       # StepRunner trait definition
│       └── shell.rs     # Shell-based step runner implementation
├── examples/
│   └── pipeline.yaml    # Example multi-cloud pipeline
└── tests/
    └── integration.rs   # End-to-end pipeline tests
```

---

## ⚙️ Configuration Reference

| Field   | Type              | Required | Description                          |
|---------|-------------------|----------|--------------------------------------|
| `name`  | `string`          | ✅        | Human-readable pipeline name         |
| `steps` | `array`           | ✅        | Ordered list of steps to execute     |
| `run`   | `string`          | ✅        | Shell command to execute             |
| `env`   | `map<string>`     | ❌        | Environment variables for this step  |

---

## 🧱 Tech Stack

| Crate        | Purpose                          |
|--------------|----------------------------------|
| `tokio`      | Async runtime & process spawning |
| `clap`       | CLI argument parsing             |
| `serde_yaml` | YAML pipeline deserialization    |
| `anyhow`     | Ergonomic error handling         |

---

## 🗺️ Roadmap

- [ ] Parallel step execution via `tokio::task::JoinSet`
- [ ] Plugin system via `Box<dyn StepRunner>` dynamic dispatch
- [ ] Webhook & Slack notifications on step failure
- [ ] Pipeline templating with variable interpolation
- [ ] Native Terraform, Pulumi, and AWS CDK step runners
- [ ] Web dashboard for pipeline run history

---

## 🤝 Contributing

Contributions are welcome! Please open an issue before submitting a PR.
See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

---

## 📄 License

MIT © Ihgedas
