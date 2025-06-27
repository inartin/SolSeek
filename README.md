# SolSeek

# Solana Vanity Address Generator

Fast Rust-based vanity address generator for Solana. Generates addresses that start, end, or contain your desired patterns. Supports searching for addresses with specific patterns at both the beginning and end simultaneously.

## Performance
Tested on MacBook Pro M1 Pro 16" (8 CPU cores):
- Average speed: **~500K+ addresses/second**
- Uses all CPU cores automatically

## Installation

### Step 1: Install Rust
```bash
# On Mac/Linux:
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# On Windows: Download and run from https://rustup.rs/
```

### Step 2: Clone/Download Code
```bash
cd SolSeek
```

### Step 3: Build
```bash
cargo build --release
```

## Usage

### Start and End Pattern Matching (New!)
```bash
# Find address starting with "S01" AND ending with "SEEK"
./target/release/solana_vanity_generator --start S01 --end SEEK

# Find address only starting with pattern
./target/release/solana_vanity_generator --start S01SEEK

# Find address only ending with pattern
./target/release/solana_vanity_generator --end SEEK

# Case-insensitive search (matches "seek", "Seek", "SEEK", etc.)
./target/release/solana_vanity_generator --start SEEK --case-sensitive false

# Case-sensitive search (matches only exact "SEEK")
./target/release/solana_vanity_generator --start SEEK --case-sensitive true
```

### Usage Examples
```bash
# Specific start/end patterns
./target/release/solana_vanity_generator --start S01 --end SEEK

# Case-insensitive matching
./target/release/solana_vanity_generator --start SEEK --case-sensitive false

# Multiple patterns with position control
./target/release/solana_vanity_generator --position start S01SEEK SEEK F0RGE

```

### Example Output

#### Start and End Pattern Example
```
Solana Vanity Address Generator
Start pattern: S01
End pattern: SEEK
Mode: Both start and end patterns must match
Case sensitive: true
Using 8 CPU threads with batch size 100000

Attempts: 15.2M | Current Rate: 520.45K addr/sec | Avg Rate: 518.32K addr/sec | Running: 29s

Found matching address!
Matched pattern: start 'S01' + end 'SEEK'
Match position: start 'S01' and end 'SEEK'
Actual match: S01 ... SEEK
Public Key: S01xxx...xxxSEEK
Secret Key: 5Kxxx...xxxxx
Total attempts: 15,247,891
Time taken: 29.4 seconds
Speed: 518,234 addresses/second
```

## Command Line Options

### Pattern Matching Options
- `--start PATTERN` - Find addresses starting with PATTERN
- `--end PATTERN` - Find addresses ending with PATTERN
- `--start PATTERN1 --end PATTERN2` - Find addresses starting with PATTERN1 AND ending with PATTERN2
- `--case-sensitive true|false` - Control case sensitivity (default: true)

### Other Options
- `--position start|end|startorend|anywhere` - Control where to search for patterns
- `PATTERN1 PATTERN2 ...` - Search for multiple patterns (any position based on --position)


## Important Notes

### Valid Characters
Only use **Base58** characters in patterns:
```
123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz
```

**Invalid characters:** `0` (zero), `O` (capital O), `I` (capital i), `l` (lowercase L)

**Common substitutions:**
- Use `1` instead of `l` or `I`
- Use `o` instead of `0` 
- For "SOL" use "S01"
- For "COOL" use "C001" or "C881"

### Security
- **Save your private key immediately** - it's only shown once
- **Test with small amounts first** before using for real funds
- Generated keys are cryptographically secure

### Performance Tips
- **Shorter patterns** = faster generation
- **Longer patterns** = exponentially slower
- **4-5 characters** = reasonable time
- **6+ characters** = may take hours/days
- **Start + End patterns** = significantly slower than single patterns (multiplicative difficulty)
- **Example**: `--start S01 --end SEEK` is much harder than just `--start S01SEEK`

## Troubleshooting

### Build Errors
```bash
# Update Rust
rustup update

# Clean and rebuild
cargo clean
cargo build --release
```

### Pattern Not Found
- Try shorter patterns
- Check for invalid Base58 characters
- Be patient - longer patterns take time

### Performance Issues
- Close other applications
- Ensure you're using `--release` build
- Check CPU temperature (throttling)


## Platform Support
- ✅ **macOS** (Intel & M1)
- ✅ **Windows** (Intel & AMD)
- ✅ **Linux** (Intel & AMD)

## License
Use at your own risk. No warranty provided.

### ❤️ Support The Project

If you find `SolSeek` useful, please consider sending a donation. It is greatly appreciated!

**Solana (SOL) Address:**
```
SeekdpXKdgHskzcCmTSvmJbuGpfzhrnog5UkBdGWg5i

```
