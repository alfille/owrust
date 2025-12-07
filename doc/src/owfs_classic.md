# OWFS (1-Wire File System): A Comprehensive Overview

## Introduction

OWFS (1-Wire File System) is a comprehensive software suite designed to make Dallas Semiconductor/Maxim 1-Wire bus systems easily accessible through a simple and intuitive interface. Rather than requiring complex low-level protocol handling, OWFS presents 1-Wire devices as a virtual filesystem where device properties appear as simple files that can be read and written using standard tools.

## Core Philosophy

The fundamental design principle is to create a virtual filesystem where the unique ID of each device becomes a directory, and individual device properties are represented as simple files. This abstraction hides the complexity of device communication behind a consistent, easy-to-use interface, making monitoring and control applications straightforward to develop.

## System Architecture

### Component Overview

OWFS consists of several interconnected components that work together:

#### 1. owserver (Backend)

The owserver is the backend component that directly communicates with 1-Wire bus master hardware. It acts as a centralized server that:

- Manages direct hardware access to the 1-Wire bus
- Implements data caching for improved performance
- Supports multithreading for concurrent client access
- Arbitrates access when multiple clients need the bus
- Handles network communication on TCP port 4304 (IANA assigned)
- Can connect to other owserver instances for distributed systems

The owserver is the recommended way to access 1-Wire devices as it allows multiple programs to share access to the bus simultaneously without conflicts.

#### 2. owfs (Filesystem Client)

The owfs program mounts the 1-Wire bus as a FUSE (Filesystem in Userspace) filesystem on Linux, FreeBSD, and macOS. With owfs:

- The entire 1-Wire bus appears as a directory tree in your filesystem
- Each device gets a directory named after its unique 64-bit ID
- Device properties are files that can be read/written with normal tools
- Standard commands like `cat`, `ls`, `echo` work with 1-Wire data

For example, reading temperature becomes as simple as:
```bash
cat /mnt/1wire/10.67C6697351FF/temperature
```

**Important Note:** The owfs filesystem component itself is not recommended for production use due to known race condition issues. Use owserver with client libraries instead.

#### 3. owhttpd (Web Interface)

Provides HTTP access to the 1-Wire bus, creating a web-based interface where:

- Device data is accessible via web browser
- RESTful-style URLs map to device properties
- Useful for monitoring and simple control applications
- Can be accessed remotely over the network

#### 4. owftpd (FTP Interface)

Offers FTP access to the 1-Wire bus, treating the device structure as an FTP directory tree. This allows:

- Standard FTP clients to browse devices
- File transfer protocols to read/write device data
- Integration with FTP-aware applications

#### 5. Command-Line Tools

A suite of shell utilities provides direct command-line access:

- **owdir**: Lists devices or directory contents
- **owread**: Reads data from a specific device property
- **owwrite**: Writes data to a device property  
- **owpresent**: Tests if a device is present on the bus
- **owget**: Combination read/directory utility

These tools are useful for scripting and automation.

## The owserver Protocol

### Network Communication

The owserver protocol enables remote access to 1-Wire devices over TCP/IP networks. This allows physical separation between the bus master hardware and client applications.

### Protocol Structure

Communication uses a simple message-based protocol with binary headers:

**Message to owserver (client to server):**
```c
struct server_msg {
    int32_t version;        // Protocol version (0)
    int32_t payload;        // Payload length in bytes
    int32_t type;           // Request type
    int32_t control_flags;  // Control options
    int32_t size;           // Expected response size
    int32_t offset;         // Offset for partial reads
};
```

**Message from owserver (server to client):**
```c
struct client_msg {
    int32_t version;        // Protocol version (0)
    int32_t payload;        // Payload length in bytes
    int32_t ret;            // Return code
    int32_t control_flags;  // Control options
    int32_t size;           // Actual data size
    int32_t offset;         // Offset for partial reads
};
```

After each header, the actual payload data is transmitted as a binary stream.

### Message Types

The protocol supports several operation types:

1. **NOP (No Operation)**: Ping/keepalive
2. **READ**: Read device property data
3. **WRITE**: Write data to device property
4. **DIR**: List directory contents
5. **PRESENCE**: Check device presence
6. **DIRALL**: Recursive directory listing
7. **GET**: Combination read/dir operation

### Connection Modes

**Non-Persistent Connection:**
- Socket created for each message exchange
- Stateless protocol
- Simpler to implement
- Thread-safe
- Higher overhead per operation

**Persistent Connection:**
- Socket reused across multiple operations
- More efficient for repeated queries
- Requires explicit connection management
- Not inherently thread-safe (requires locking)
- Server may refuse persistent connections under load

### Control Flags

The protocol supports various flags to control behavior:

- **Temperature scale**: Celsius, Fahrenheit, Kelvin, Rankine
- **Pressure scale**: Millibar, atmosphere, mmHg, PSI, Pascal
- **Device format**: Different display formats for device IDs
- **Caching control**: Force uncached reads
- **Persistence**: Request persistent connection

## Virtual Filesystem Structure

### Root Directory Structure

The OWFS virtual filesystem has a well-defined structure:

```
/
├── 10.67C6697351FF/          # Temperature sensor (DS18S20)
│   ├── temperature           # Current temperature
│   ├── temphigh              # High alarm threshold
│   ├── templow               # Low alarm threshold
│   ├── family                # Device family code
│   ├── id                    # Unique device ID
│   └── type                  # Device type name
├── 26.A2D1B2000000/          # Battery monitor (DS2438)
│   ├── temperature
│   ├── VAD                   # A/D voltage
│   ├── VDD                   # Supply voltage
│   └── vis                   # Humidity sensor voltage
├── /bus.0/                   # First bus master
├── /uncached/                # Force fresh reads
├── /alarm/                   # Devices in alarm state
├── /simultaneous/            # Trigger simultaneous operations
├── /statistics/              # Bus statistics
└── /settings/                # Runtime configuration
    ├── timeout/
    ├── units/
    └── return_codes/
```

### Device Directories

Each device appears as a directory named with its unique 64-bit identifier, displayed in the format: `family.id` (e.g., `10.67C6697351FF`).

Inside each device directory, files represent device properties:
- Read-only properties (like family code, type)
- Read-write properties (like alarm thresholds, switch states)
- Volatile properties (like temperature, that change frequently)

### Special Directories

**uncached/**: Forces fresh reads from devices, bypassing cache. Useful when real-time data is critical.

**alarm/**: Contains devices currently in alarm state. Devices can be configured with thresholds; when exceeded, they appear here.

**simultaneous/**: Allows triggering simultaneous operations across multiple devices, such as starting temperature conversions on all sensors at once.

**statistics/**: Provides runtime statistics about bus operations, errors, and performance metrics.

**settings/**: Allows dynamic configuration changes including timeouts, temperature scales, and caching behavior.

## Supported Hardware

### Bus Masters

OWFS supports a wide variety of bus master adapters:

#### USB Adapters
- **DS9490R/DS9490B**: The most common USB adapter based on the DS2490 chip
- **LinkUSB**: FTDI-based USB adapter with improved timing
- **ECLO**: Alternative USB bus master

#### Serial Adapters
- **DS9097U**: Active serial port adapter with DS2480B chip
- **DS9097**: Passive serial adapter (requires software bit-banging)
- **HA5**: Multidrop ASCII protocol adapter
- **DS1410E**: Parallel port adapter

#### I2C Adapters
- **DS2482-100**: Single-channel I2C to 1-Wire bridge
- **DS2482-800**: Eight-channel I2C to 1-Wire bridge (creates 8 independent buses)

#### Network Adapters
- **HA7Net**: Ethernet-enabled 1-Wire adapter
- **OW-SERVER-ENET**: Ethernet 1-Wire server
- **EtherWeather**: Network weather station interface

#### Linux Kernel Support
- **w1 kernel module**: Native Linux kernel driver for various GPIO-based and built-in bus masters

#### Remote Connections
- **owserver**: Connect to another owserver instance over TCP/IP
- Allows distributed architectures and remote hardware access

### Simulated Adapters

For development and testing:
- **Fake adapter**: Simulates devices without hardware
- **Tester adapter**: Generates predictable test patterns
- **Mock adapter**: Random data for stress testing

## Caching System

### Cache Architecture

OWFS implements intelligent caching to improve performance by reducing slow 1-Wire bus operations:

#### Cache Categories

1. **Volatile Properties**: Data that changes frequently (e.g., temperature readings)
   - Short cache lifetime (default: few seconds)
   - Automatically expired after timeout

2. **Stable Properties**: Data that rarely changes (e.g., device family, serial number)
   - Long cache lifetime (default: several minutes)
   - Reduces redundant queries

3. **Presence Information**: Device presence on the bus
   - Separate timeout for presence checks
   - Prevents unnecessary device searches

### Cache Location

In owserver-based architectures, caching occurs in the server, not clients. This means:
- All clients benefit from cached data
- Reduced bus traffic from multiple simultaneous clients
- Centralized cache management
- Consistent data across clients

### Cache Control

Users can control caching behavior:
- Access `/uncached/` directory to force fresh reads
- Adjust timeout values dynamically via `/settings/timeout/`
- Configure cache behavior at startup

## Language Bindings

### Full Library Support (libow)

OWFS provides complete library support for several languages through libow:

- **owcapi**: C API for direct library access
- **owperl**: Perl bindings
- **owtcl**: Tcl bindings
- **owphp**: PHP bindings

These bindings link directly to the OWFS library, providing full functionality and performance.

### Lightweight Support (OWNet)

OWNet provides lightweight network-based access:

- **OWNet.py**: Python module
- **OWNet.pm**: Perl module
- **OWNet.php**: PHP module
- **OWNet.vb**: Visual Basic module
- **ownet**: C implementation

OWNet modules communicate with owserver via the network protocol, making them:
- Independent of the full OWFS library
- Easy to deploy
- Suitable for embedded systems
- Network-capable by design

## Configuration

### Configuration File

OWFS components can be configured via `/etc/owfs.conf`:

```ini
######################## SOURCES ########################
# Connect clients to local owserver
! server: server = localhost:4304

# owserver connects to USB hardware
server: usb = all

######################### OWFS ##########################
mountpoint = /mnt/1wire
allow_other

####################### OWHTTPD #########################
http: port = 2121

####################### OWFTPD ##########################
ftp: port = 2120

####################### OWSERVER ########################
server: port = localhost:4304
```

### Command-Line Options

All OWFS programs support extensive command-line options:

**Temperature Scales:**
- `-C` or `--Celsius` (default)
- `-F` or `--Fahrenheit`
- `-K` or `--Kelvin`
- `-R` or `--Rankine`

**Pressure Scales:**
- `--mbar` (default)
- `--atm`, `--mmHg`, `--inHg`, `--psi`, `--Pa`

**Device ID Format:**
- `-f` family.id.crc format (default)
- `-fi` family.id format
- `-fc` family.crc format

**Caching:**
- `--timeout_volatile=<seconds>`: Cache timeout for changing data
- `--timeout_stable=<seconds>`: Cache timeout for static data
- `--timeout_presence=<seconds>`: Device presence cache timeout

**Network:**
- `-s <address:port>`: Connect to owserver
- `-p <port>`: Port for owserver to listen on

**Hardware:**
- `-u` or `--usb`: Use USB adapter
- `-d <device>`: Serial port device
- `--i2c=<address>`: I2C adapter
- `--ha7net=<address>`: HA7Net network adapter

## Typical Deployment Scenarios

### Scenario 1: Single Machine, Local Access

```
┌─────────────────────────┐
│   Application           │
├─────────────────────────┤
│   owfs (FUSE mount)     │
├─────────────────────────┤
│   owserver              │
├─────────────────────────┤
│   USB Bus Master        │
└─────────────────────────┘
         │
    1-Wire Bus
```

All components run on one machine. Applications access devices through the filesystem mount.

### Scenario 2: Network Architecture

```
┌─────────────────┐      ┌─────────────────┐
│  Client App     │      │  Web Browser    │
│  (owread/write) │      │  (owhttpd)      │
└────────┬────────┘      └────────┬────────┘
         │                        │
         └──────────┬─────────────┘
                    │ TCP/IP
         ┌──────────▼────────────┐
         │     owserver          │
         ├───────────────────────┤
         │  USB Bus Master       │
         └───────────────────────┘
                    │
              1-Wire Bus
```

owserver provides network access. Multiple clients can connect from different machines.

### Scenario 3: Distributed System

```
┌─────────────┐
│   Client    │
└──────┬──────┘
       │ TCP/IP
┌──────▼──────┐       ┌─────────────┐
│  owserver1  │───────│  owserver2  │
│  (Master)   │TCP/IP │  (Slave)    │
└─────┬───────┘       └──────┬──────┘
      │                      │
  1-Wire Bus            1-Wire Bus
   (Indoor)              (Outdoor)
```

Multiple owserver instances can be chained, with one acting as master and aggregating data from slave servers managing different physical buses.

## Performance Considerations

### Caching Impact

Proper cache configuration dramatically improves performance:
- Cached reads are nearly instantaneous
- Uncached reads require full 1-Wire bus transactions (milliseconds to seconds)
- For frequently-read data, caching reduces bus load by orders of magnitude

### Bus Master Selection

Different bus masters have different performance characteristics:
- USB adapters (DS9490R) are generally fastest
- I2C adapters (DS2482) offer good performance
- Serial adapters vary widely (DS9097U is much faster than passive DS9097)
- Network adapters add network latency but enable remote access

### Multithreading

owserver's multithreaded design allows:
- Concurrent client connections
- Parallel queries to different devices
- Asynchronous operations where possible

However, the 1-Wire bus itself is fundamentally serial, so there are inherent limits to parallelization.

## Security Considerations

### Access Control

**Filesystem Access (owfs):**
- Standard Unix permissions apply
- `allow_other` option needed for multi-user access
- Requires proper FUSE configuration

**Network Access (owserver):**
- No built-in authentication in the protocol
- Should be protected by firewalls
- Consider SSH tunneling for remote access
- VPN recommended for public network exposure

### Permission Requirements

- Serial port access requires appropriate user permissions or root
- USB devices need proper udev rules or user groups
- I2C typically requires root or i2c group membership
- FUSE mounting may require specific permissions

## Advantages of OWFS

1. **Simplicity**: File-based interface is intuitive and requires no special libraries for basic use
2. **Flexibility**: Multiple access methods (filesystem, HTTP, FTP, network, command-line)
3. **Standardization**: Consistent interface regardless of device type
4. **Scriptability**: Easy integration with shell scripts, cron jobs, and standard tools
5. **Network-enabled**: Built-in remote access capabilities
6. **Multi-platform**: Runs on Linux, BSD, macOS, and Windows (via Cygwin)
7. **Language support**: Bindings for many programming languages
8. **Caching**: Intelligent caching improves performance
9. **Open source**: Free software under GPL license
10. **Mature**: Over two decades of development and real-world deployment

## Limitations and Considerations

1. **Performance**: File-based abstraction adds overhead compared to direct hardware access
2. **Filesystem component**: owfs itself has known race conditions; owserver is preferred
3. **Security**: Network protocol lacks built-in authentication
4. **Learning curve**: While conceptually simple, the full system is complex
5. **Bus limitations**: Inherits all limitations of 1-Wire (speed, topology, noise sensitivity)
6. **Dependency**: Requires FUSE kernel support for filesystem features

## Common Use Cases

### Home Automation
- Temperature monitoring throughout a house
- HVAC control based on sensor readings
- Light and appliance control
- Security system integration

### Environmental Monitoring
- Weather stations
- Greenhouse monitoring and control
- Aquarium automation
- Server room temperature monitoring

### Industrial Applications
- Process control and monitoring
- Equipment temperature tracking
- Access control systems
- Data logging

### Hobbyist Projects
- Custom sensor networks
- Home weather stations
- Learning embedded systems
- Prototype development

## Getting Started

### Installation

Most Linux distributions package OWFS:

**Debian/Ubuntu:**
```bash
sudo apt-get install owserver ow-shell owhttpd owfs
```

**From Source:**
```bash
./configure
make
sudo make install
```

### Basic Setup

1. **Connect hardware**: Plug in USB adapter or configure serial port
2. **Start owserver**: 
   ```bash
   owserver -u --error_level=3
   ```
3. **Test connection**:
   ```bash
   owdir
   ```
4. **Read device**:
   ```bash
   owread /10.67C6697351FF/temperature
   ```

### Simple Script Example

```bash
#!/bin/bash
# Read all DS18B20 temperature sensors

for device in $(owdir | grep "^10\."); do
    temp=$(owread /${device}/temperature)
    echo "Device $device: ${temp}°C"
done
```

## Troubleshooting

### Debug Output

Enable debugging to diagnose issues:
```bash
owserver -u --error_level=9 --foreground
```

### Common Issues

**No devices found:**
- Check bus master connection
- Verify device wiring
- Ensure proper pull-up resistor (4.7kΩ typical)
- Check for shorts or breaks in bus

**Read errors:**
- Bus too long (use shorter cables or active components)
- Electrical noise (add filtering, better grounding)
- Insufficient power (use external power for parasitic devices)
- Timing issues (try different bus master or lower speed)

**Permission denied:**
- Add user to appropriate group (dialout for serial, plugdev for USB)
- Configure udev rules for USB devices
- Run with elevated privileges (not recommended for production)

## Conclusion

OWFS transforms the complex task of 1-Wire device communication into simple file operations. By presenting devices as a virtual filesystem and providing network-based access through owserver, it enables easy integration of 1-Wire sensors and devices into monitoring, control, and automation systems. While the filesystem abstraction adds some overhead, the resulting simplicity and flexibility make OWFS an excellent choice for most 1-Wire applications, from hobby projects to industrial deployments.

The combination of multiple access methods, comprehensive language support, intelligent caching, and a mature, well-documented codebase has made OWFS the de facto standard for 1-Wire access on Unix-like systems for over 20 years.
