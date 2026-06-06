# RustRiff
## From Rust to Rock in miliseconds!
**RustRiff** is a desktop guitar amplifier built with rust (**React + Typescript **, **Rust + Tauri**).
RustRiff models core amp controls (gain, tone stack, channel flow), an effect chain, and cabinet 
impulse-response (IR).

<p align="center">
  <a href="https://zacharyvds-is.github.io/Guitar-Amplifier/"><img alt="Docs" src="https://img.shields.io/badge/Docs-GitHub%20Pages-2563EB?style=for-the-badge"></a>
  <a href="https://zacharyvds-is.github.io/Guitar-Amplifier/frontend/index.html"><img alt="Frontend API" src="https://img.shields.io/badge/API-Frontend%20TypeDoc-0EA5E9?style=for-the-badge"></a>
  <a href="https://zacharyvds-is.github.io/Guitar-Amplifier/backend/doc/rustriff_lib/index.html"><img alt="Backend API" src="https://img.shields.io/badge/API-Backend%20Rustdoc-7C3AED?style=for-the-badge"></a>
  <a href="https://github.com/ZacharyVds-IS/Guitar-Amplifier"><img alt="Repository" src="https://img.shields.io/badge/Repository-GitHub-111827?style=for-the-badge"></a>
</p>

## Get Started

### Prerequisites

- Node.js 24+
- npm 10+
- Rust stable toolchain
- Tauri system dependencies for your OS

### Before running (windows development only!)
On windows our project utilizes ASIO drivers for low latency audio processing, which requires some additional configuration.

#### LLVM  (cpal uses bindgen to bridge ASIO sdk C++ code to Rust)
Installation of LLVM is simple using winget, just execute the following command in your terminal:
```powershell
winget install LLVM.LLVM
```
 after running the comand make sure to restart your terminal
Now we need to add LIBCLANG to our environment variables this can be done with the following command
```powershell
setx LIBCLANG_PATH "C:\Program Files\LLVM\bin"
```
#### Desktop Development with C++ workload
This can be installed via **Visual Studio** [Installer](https://visualstudio.microsoft.com/downloads/).

or download this toolchain with this command:
```powershell
winget install Microsoft.VisualStudio.2022.BuildTools --override "--add Microsoft.VisualStudio.Workload.VCTools --includeRecommended --passive"
```
#### ASIO SDK Driver
Lastly for builds to succeed we will need to have the ASIO SDK installed, you can download it from the [Steinberg website](https://www.steinberg.net/developers/)
Place the contents of this new folder inside a known location for ex . ```C:\ASIO_SDK```

Example structure:
```
C:\ASIO_SDK
├── asio
├── common
├── driver
├── host
├── ...
```

Lastly add the folder to your environment variables with the following name: ```CPAL_ASIO_DIR``` the value in the example would be ```C:\ASIO_SDK```

### Running locally
```powershell
npm install
npm run tauri dev
```
### Building for production
```powershell
npm install
npm run tauri build
```

### Work on documentation
RustRiff docs is a combination of custom written markdown and auto generated api references.
```powershell
//Development docs run
npm run docs:dev

//Building the combined documentation
npm run docs:build
```

## License
See `LICENSE.md`.
