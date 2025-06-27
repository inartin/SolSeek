# SolSeek

# Solana Vanity Address Generator

CPU Rust-based vanity address generator for Solana.
Generates addresses that start, end, or contain your desired patterns (letters or numbers).

## Features
- Logs private key
- Optional case sensitive
- Find patterns at: Start, End, Contain, Start and End, Start or End

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

## Usage Examples

```bash
# Find address starting with "SO1" AND ending with "SEEK"
./run.sh --start SO1 --end SEEK
# OR: ./target/release/solana_vanity_generator --start SO1 --end SEEK

# Find address only starting with pattern
./run.sh --start SO1SEEK

# Find address only ending with pattern
./run.sh --end SEEK

# Case-sensitive search (matches only exact "SEEK")
./run.sh --start SEEK --case-sensitive true

# Multiple patterns with position control
./run.sh --position start SO1SEEK SEEK F0RGE
```

### Example Output

#### Start and End Pattern Example
```
Solana Vanity Address Generator
Start pattern: SO1
End pattern: SEEK
Mode: Both start and end patterns must match
Case sensitive: true
Using 8 CPU threads with batch size 100000

Attempts: 15.2M | Current Rate: 520.45K addr/sec | Avg Rate: 518.32K addr/sec | Running: 29s

Found matching address!
Matched pattern: start 'SO1' + end 'SEEK'
Match position: start 'SO1' and end 'SEEK'
Actual match: SO1 ... SEEK
Public Key: SO1xxx...xxxSEEK
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
- For "SOL" use "SO1"
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
- **Example**: `--start SO1 --end SEEK` is much harder than just `--start SO1SEEK`

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

### Launch Script
The `run.sh` script is a convenience wrapper that auto-builds and forwards arguments. Use `./run.sh` instead of the full `./target/release/solana_vanity_generator` command.

### ❤️ Support The Project

If you find `SolSeek` useful, please consider sending a donation. It is greatly appreciated!

**Solana (SOL) Address:**
```
SeekdpXKdgHskzcCmTSvmJbuGpfzhrnog5UkBdGWg5i
```
