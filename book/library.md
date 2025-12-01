# Owrust Library

## **🏗️ Architectural Overview**

The design is centered around three main components:

1. **OwClient (The Configuration and API):** Manages connection settings and exposes public methods (read, write, dir).  
2. **OwMessageSend / OwMessageReceive (Protocol Structs):** Data structures that map directly to the owserver network packet format.  
3. **Network Functions (send\_packet, get\_packet):** Handle TCP/IP connection, byte serialization, and deserialization.

### **1\. The OwClient Structure**

The **OwClient** is the main entry point for the user. It stores all configuration settings that influence how the owserver should process requests.

| Field | Purpose | Default Value |
| :---- | :---- | :---- |
| **owserver** | The network address (IP:Port) of the daemon. | "localhost:4304" |
| **temperature, pressure, format** | User preferences for output units and 1-Wire ID display format. | DEFAULT (Celsius/mBar/FdI) |
| **flag** | A **32-bit integer** where all enum preferences (units, format) are encoded as bit flags, sent in the message header. | Calculated in make\_flag() |
| **hex, bare, slash** | Display and output control options. | false |

* **make\_flag():** This essential private method takes the high-level Rust Enum settings and translates them into the required **bitwise flags** for the owserver protocol. For instance, setting Temperature::FARENHEIT contributes the 0x00010000 bit to the final flag value.

---

## **📦 Protocol and Communication Flow**

The library strictly implements the **owserver protocol**, which relies on a fixed-size header and variable-length payload transmitted over a TCP stream.

### **A. Message Structures**

* **OwMessageSend:** Represents an outgoing message. It includes fields for version, mtype (command, e.g., READ=2, WRITE=3), flags, and payload (length of the content).  
  * **Path Handling:** The add\_path method is crucial, using std::ffi::CString::new(path).into\_bytes\_with\_nul() to ensure the path is correctly **nul-terminated**, a requirement for C-based networking protocols like owserver.  
* **OwMessageReceive:** Represents an incoming response. It includes the status code in ret and the length of the data in payload. It uses u32::from\_be\_bytes to parse the header, confirming the protocol uses **Big Endian** (network byte order).

### **B. Network Handling**

1. **send\_packet:** Opens a **TCP connection** to the configured owserver address. It serializes the 24-byte header fields into big-endian bytes, prepends the header to the payload, and sends the complete packet via stream.write\_all().  
2. **get\_packet:** Reads the incoming 24-byte header, checks for **ping messages** (indicated by payload \< 0), and then reads the variable-length content payload if one is expected.  
3. **get\_msg\_many:** Handles directory commands (DIR), where the server may send **multiple packets** to transmit the full directory listing. This function loops to collect all packets, concatenating their content and replacing the terminating null of each directory entry with a comma (,) for proper formatting.

---

## **🚨 Error Handling and Utilities**

### **OwError (Custom Error Type)**

The original code's error type has been improved (as shown in the provided code) from a simple Enum to a **struct (OwError) that includes a String field (details)**. This is good practice in Rust, as it allows the library to attach detailed contextual information (like network error messages or bad path arguments) directly to the error object.

### **Data Manipulation**

* **Public API:** The public functions like **read**, **write**, **dir**, **present**, and **size** are implemented efficiently by internally calling private make\_\* and send\_get\_\* functions.  
* **show\_result / show\_text:** These utilities format the raw Vec\<u8\> response data either as a string (show\_text) or as space-separated hexadecimal bytes (show\_result when self.hex is true).  
* **input\_to\_write:** Handles converting the user's input string into the byte vector needed for a WRITE operation. The error check if \! s.len().is\_multiple\_of(2) correctly enforces that hexadecimal input must have an even length.