# XION Social Payment Smart Contract

A CosmWasm smart contract for the XION blockchain, providing a comprehensive social payment platform with user management, friends system, and multi-type payment processing. Includes robust username registration, wallet mapping, and proof verification (including zkTLS support).

---

## ✨ Features

- **User Management**

  - Register unique usernames (case-insensitive, 3-50 chars, alphanumeric + underscores)
  - Map wallet addresses to usernames and vice versa
  - Update user profiles (display name, profile picture)
  - Search users by username or display name

- **Friends System**

  - Send, accept, and decline friend requests
  - Remove friends
  - Query friends list and pending requests

- **Payment System**

  - Direct payments between friends
  - Payment requests and help/crowdfunding requests
  - Escrow system for secure payments
  - Proof submission and verification (Photo, Document, Location, zkTLS, Manual)
  - Payment approval, rejection, and cancellation

- **Events & Queries**

  - Emits events for key actions (e.g., `username_registered`)
  - Query endpoints for all user, friend, and payment data

- **Security & Validation**
  - Strict username validation and uniqueness
  - Case-insensitive lookups
  - Comprehensive error handling

---

## 🛠️ Tech Stack

- **CosmWasm** (for smart contracts on Cosmos chains)
- **Rust** (safe, performant contract logic)
- **WASM** (compiled output)

---

## 📦 Usage

### Build

```sh
cargo wasm
```

### Test

```sh
cargo test
```

### Generate JSON Schemas

```sh
cargo run --example schema
```

### Directory Structure

```
src/
  contract.rs        # Main contract logic
  state.rs           # Data structures and storage maps
  msg.rs             # API message and response types
  error.rs           # Error definitions
  helpers.rs         # Utility functions
  integration_tests.rs # Comprehensive test suite
artifacts/
  cw_counter.wasm    # Compiled WASM binary
schema/
  *.json             # Generated JSON schemas for API
```

---

## API Overview

### Execute Messages

- `RegisterUser { username, display_name }` — Register a new user with a unique username
- `UpdateUserProfile { display_name, profile_picture }` — Update your display name or profile picture
- `SendFriendRequest { to_username }` — Send a friend request to another user
- `AcceptFriendRequest { from_username }` — Accept a pending friend request
- `DeclineFriendRequest { from_username }` — Decline a pending friend request
- `RemoveFriend { username }` — Remove a user from your friends list
- `SendDirectPayment { to_username, amount, description, proof_type }` — Send a direct payment to a friend
- `CreatePaymentRequest { to_username, amount, description, proof_type }` — Request a payment from another user
- `CreateHelpRequest { to_username, amount, description, proof_type }` — Create a help/crowdfunding request
- `SubmitProof { payment_id, proof_data }` — Submit proof for a payment or help request
- `ApprovePayment { payment_id }` — Approve a payment after proof submission
- `RejectPayment { payment_id }` — Reject a payment after proof submission
- `CancelPayment { payment_id }` — Cancel a pending payment

### Query Messages

- `GetUserByUsername { username }` — Get user profile by username
- `GetUserByWallet { wallet_address }` — Get user profile by wallet address
- `IsUsernameAvailable { username }` — Check if a username is available
- `SearchUsers { query }` — Search users by username or display name
- `GetUsernameByWallet { wallet_address }` — Get username for a wallet address
- `GetWalletByUsername { username }` — Get wallet address for a username
- `HasUsername { wallet_address }` — Check if a wallet has a registered username
- `GetUserFriends { username }` — Get a user's friends list
- `GetPendingRequests { username }` — Get pending friend requests for a user
- `AreFriends { username1, username2 }` — Check if two users are friends
- `GetPaymentById { payment_id }` — Get payment details by ID
- `GetPaymentHistory { username }` — Get payment history for a user
- `GetPendingPayments { username }` — Get pending payments for a user

### Events

- `username_registered` — Emitted when a user successfully registers a username. Attributes: `wallet`, `username`
- `register_user` — Emitted on user registration. Attributes: `username`, `wallet`
- `update_user_profile` — Emitted when a user updates their profile. Attributes: `username`
- `send_friend_request` — Emitted when a friend request is sent. Attributes: `from_username`, `to_username`
- `accept_friend_request` — Emitted when a friend request is accepted. Attributes: `from`, `to`
- `decline_friend_request` — Emitted when a friend request is declined. Attributes: `from`, `to`
- `remove_friend` — Emitted when a friend is removed. Attributes: `user`, `removed_friend`
- `send_direct_payment` — Emitted when a direct payment is sent. Attributes: `from`, `to`, `payment_id`, `amount`
- `create_payment_request` — Emitted when a payment request is created. Attributes: `from`, `to`, `payment_id`, `amount`
- `create_help_request` — Emitted when a help/crowdfunding request is created. Attributes: `from`, `to`, `payment_id`, `amount`
- `submit_proof` — Emitted when proof is submitted for a payment. Attributes: `payment_id`, `submitter`
- `approve_payment` — Emitted when a payment is approved. Attributes: `payment_id`, `approver`
- `reject_payment` — Emitted when a payment is rejected. Attributes: `payment_id`, `rejector`
- `cancel_payment` — Emitted when a payment is cancelled. Attributes: `payment_id`, `canceller`

---

## 📄 License

MIT

---

## 🙋‍♂️ Contact

Questions or ideas? Open an issue or reach out on [GitHub](https://github.com/kyisaiah47).
