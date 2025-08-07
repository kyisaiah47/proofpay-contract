# Proof of Work Smart Contract

A **CosmWasm smart contract** written in Rust for trustless, on-chain freelance jobs.  
Clients can post jobs, workers can submit proofs, and clients can accept/reject work—all managed transparently on the blockchain.

---

## ✨ Features

- **Post Jobs:** Anyone can create a new job with a description.
- **Submit Proof:** Workers submit on-chain proof of completion for specific jobs.
- **Client Acceptance:** Only the original client can accept submitted proof and mark jobs as completed.
- **Full Query Support:** List all jobs or fetch individual job details by ID.
- **Ownership:** Contract owner saved at initialization for admin operations.

---

## 🛠️ Tech Stack

- **CosmWasm** (for smart contracts on Cosmos chains)
- **Rust** (safe, performant contract logic)
- **WASM** (compiled output)

---

## 📦 Usage

### Build

```bash
cargo wasm
```

### Deploy

Upload the compiled WASM to your preferred CosmWasm-compatible blockchain using your favorite CLI or dApp.

### Execute

- **Post a Job:**  
  `ExecuteMsg::PostJob { description }`
- **Submit Proof:**  
  `ExecuteMsg::SubmitProof { job_id, proof }`
- **Accept Proof:**  
  `ExecuteMsg::AcceptProof { job_id }`

### Query

- **Get Job Details:**  
  `QueryMsg::GetJob { job_id }`
- **List All Jobs:**  
  `QueryMsg::ListJobs {}`

---

## 📝 Example

```json
// Post a new job
{
  "post_job": {
    "description": "Design a landing page for our new product."
  }
}

// Submit proof of work
{
  "submit_proof": {
    "job_id": 1,
    "proof": "https://link.to/my/proof.png"
  }
}

// Accept the proof (client only)
{
  "accept_proof": {
    "job_id": 1
  }
}

// Query a job
{
  "get_job": {
    "job_id": 1
  }
}
```

---

## 📄 License

Apache-2.0

---

## 🙋‍♂️ Contact

Questions or ideas? Open an issue or reach out on [GitHub](https://github.com/kyisaiah47).
