# Username Management Functionality

This document describes the enhanced username management features added to the XION smart contract.

## Overview

The contract now provides comprehensive username management with case-insensitive validation, preventing username conflicts and providing easy wallet-to-username mapping.

## State Variables (Equivalent)

The contract stores username mappings using CosmWasm storage:

- `USERS_BY_USERNAME: Map<String, User>` - Maps normalized usernames to User structs
- `USERS_BY_WALLET: Map<Addr, String>` - Maps wallet addresses to usernames

## Username Functions

### Execute Messages

#### `RegisterUser`

```rust
RegisterUser {
    username: String,
    display_name: String
}
```

- Registers a username for the calling wallet
- Validates username format (3-50 characters, alphanumeric + underscores)
- Performs case-insensitive checking (converts to lowercase)
- Prevents duplicate usernames and multiple usernames per wallet
- Emits `username_registered` event

#### Username Validation Rules

- **Length**: 3-50 characters
- **Characters**: Alphanumeric (a-z, A-Z, 0-9) and underscores (\_) only
- **Case Sensitivity**: Case-insensitive (stored in lowercase)
- **Uniqueness**: One username per wallet, no duplicate usernames
- **Empty Check**: Prevents empty usernames

### Query Messages

#### `IsUsernameAvailable`

```rust
IsUsernameAvailable { username: String }
```

Returns: `UsernameAvailableResponse { available: bool }`

- Checks if a username is available for registration
- Validates format first (invalid formats return `false`)
- Case-insensitive checking

#### `GetUsernameByWallet`

```rust
GetUsernameByWallet { wallet_address: String }
```

Returns: `UsernameResponse { username: String }`

- Returns the username for a given wallet address
- Errors if wallet has no registered username

#### `GetWalletByUsername`

```rust
GetWalletByUsername { username: String }
```

Returns: `WalletResponse { wallet_address: String }`

- Returns the wallet address for a given username
- Case-insensitive username lookup
- Errors if username doesn't exist

#### `HasUsername`

```rust
HasUsername { wallet_address: String }
```

Returns: `HasUsernameResponse { has_username: bool }`

- Checks if a wallet address has a registered username
- Returns `true` if registered, `false` otherwise

#### `GetUserByUsername`

```rust
GetUserByUsername { username: String }
```

Returns: `UserResponse { user: User }`

- Returns full user profile by username
- Case-insensitive lookup

## Events

### `username_registered`

Emitted when a user successfully registers a username:

```rust
Event::new("username_registered")
    .add_attribute("wallet", wallet_address)
    .add_attribute("username", normalized_username)
```

## Integration with Existing Features

### Friend System

- All friend requests now use normalized usernames
- Case-insensitive friend lookups
- Consistent username handling across all social features

### Payment System

- Payments work with usernames (case-insensitive)
- Username validation in payment functions
- Consistent user identification

### User Management

- Enhanced user registration with username normalization
- All user queries support case-insensitive username lookup
- Consistent username storage and retrieval

## API Examples

### Register a Username

```json
{
	"register_user": {
		"username": "Alice_123",
		"display_name": "Alice Smith"
	}
}
```

### Check Username Availability

```json
{
	"is_username_available": {
		"username": "alice_123"
	}
}
```

### Get Username by Wallet

```json
{
	"get_username_by_wallet": {
		"wallet_address": "xion1abc..."
	}
}
```

### Get Wallet by Username

```json
{
	"get_wallet_by_username": {
		"username": "alice_123"
	}
}
```

### Check if Wallet Has Username

```json
{
	"has_username": {
		"wallet_address": "xion1abc..."
	}
}
```

## Response Types

All response types are fully documented in the generated JSON schemas:

- `username_response.json`
- `wallet_response.json`
- `has_username_response.json`
- `username_available_response.json`

## Testing Coverage

The contract includes comprehensive tests for:

- ✅ Case-insensitive username registration
- ✅ Username format validation (length, characters)
- ✅ Duplicate prevention
- ✅ All new query functions
- ✅ Username availability checking with validation
- ✅ Integration with existing social payment features

## Security Features

- **Input Validation**: Strict username format validation
- **Duplicate Prevention**: Case-insensitive uniqueness checking
- **Access Control**: One username per wallet address
- **Consistent Storage**: Normalized username storage prevents conflicts
- **Error Handling**: Comprehensive error messages for all edge cases

## XION Integration

The username system works seamlessly with XION's account abstraction:

- Direct wallet-to-username mapping
- Support for XION address formats
- Compatible with XION's smart account features
- Ready for integration with XION's social recovery systems
