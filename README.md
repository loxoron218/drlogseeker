# drlogseeker

[![License: GPLv3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)
[![Built with Vibe](https://img.shields.io/badge/Vibe-Coded-ff69b4)](https://cursor.so)

**GTK4/Libadwaita/Rust tool for analyzing DR logs from MAAT DROffline and foobar2000**  
*"Because sometimes you don't have time to wait for a torrent tracker invite"*

![drlogseeker Interface](https://github.com/user-attachments/assets/aae0a156-b317-4e57-8946-1d2f5bd9c99a)

## Features
- **Dual-Language DR Detection** - Finds both English/Russian DR log formats
- **Color-Coded for Your Sanity** - Full-spectrum color mapping of DR values (0-14)
- **Parallelized Analysis** - Rayon-powered scanning for massive log collections
- **Filesystem Integration** - Optional destructive operations for clean workflows
- **Keyboard Controls** - Keyboard-driven interface with vim-like controls

## Installation

### Arch Linux (AUR) (Coming Soon)
```bash
yay -S drlogseeker
```

### Flatpak (Coming Soon)
```bash
flatpak install flathub com.github.drlogseeker
```

## DR Chromesthesia System

| DR Range | Color       | Audio Health         |
|----------|-------------|----------------------|
| 0-7      | Red      | Very Compressed      |
| 8        | Burnt    | Highly Compressed    |
| 9        | Orange   | Compressed           |
| 10       | Yellow   | Slightly Compressed  |
| 11       | Lime     | Moderate             |
| 12       | Mint     | Good                 |
| 13       | Green    | Very Good            |
| 14       | Neon     | Excellent            |

## Basic Usage
1. **Search Logs**  
   Type in your favorite album on Nicotine+
2. **Filter Text Files**  
   Press `Ctrl+F` to search and choose 'Text' from the filetype menu. Then press `Ctrl+A` to select all files and download them
3. **Organize Downloads**  
   It's recommended to turn on the setting to save downloads in subfolders for each user. This improves usability by helping you identify which user a log belongs to after scanning.
4. **Batch Analyze**  
   Open drlogseeker and pick the folder all the files *.txt/log are located in and scan

## Contributing
All contributions are welcomed, especially:
- GTK4 CSS wizards
- i18n/Russian language specialists
- AUR/Fedora/Flatpak packaging experts

## Related Projects
- [DR Loudness War](https://dr.loudness-war.info/) - DR database
- [Nicotine+](https://nicotine-plus.org/) - Open source Soulseek client
- [Soulseek](https://www.slsknet.org/) - Official P2P file-sharing network
