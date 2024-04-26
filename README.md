# WahWah VST Plugin

## Motivation
The WahWah VST plugin emulates the iconic sound of a Wah-Wah pedal, such as the renowned Cry-Baby. This plugin automates the wah-wah effect, freeing musicians from the need to manually control the pedal during performances. It's designed for ease of use, allowing artists to focus more on their creativity and performance.

## Applications
This plugin is primarily aimed at guitarists but is versatile enough for live or studio use with other instruments like pianos. Being VST compatible, it integrates seamlessly into Digital Audio Workstations (DAWs), enhancing the creative possibilities across various musical genres.

## Features

### Functionality
- **Automated Pedal Control**: Aligns the wah-wah effect with the onsets of musical phrases, eliminating the need for manual pedal adjustments.
- **Versatile Sound Manipulation**: Users can tailor the sound with various adjustable parameters to fit their musical style and preference.

### [Demo Video](https://youtu.be/6olo_rhbeC8)

### Parameters
| Parameter                | Description                | Range         | Default Value |
|--------------------------|----------------------------|---------------|---------------|
| Gain                     | Adjusts the input signal level. | 0.0 to 1.0    | 0.0           |
| Envelope Attack Rate     | Controls the responsiveness of the effect to changes in input. | 0.0001 to 0.1 | 0.001         |
| Envelope Decay Rate      | Controls how quickly the effect fades after input ceases. | 0.0001 to 0.01 | 0.0005       |
| Onset Threshold          | Sets the sensitivity for detecting the start of musical notes. | 0.0 to 1.0    | 0.15          |
| Reset Threshold          | Determines the level at which the effect resets. | 0.0 to 1.0    | 0.05          |
| Use Onset Detection      | Enables or disables automatic detection of note beginnings. | Boolean       | false         |
| LFO Frequency            | Frequency of the Low-Frequency Oscillator, which modulates the filter. | 0.0 to 100.0 | 4.0           |
| LFO Intensity            | Depth of the filter modulation. | 0.0 to 4000.0 | 100.0         |
| Bandpass Low Frequency   | Sets the lower boundary of the filter's frequency range. | 0.0 to 9600.0 | 100.0         |
| Bandpass High Frequency  | Sets the upper boundary of the filter's frequency range. | 0.0 to 9600.0 | 3000.0        |

## Installation

### Building the VST Plugin
Compile and bundle the plugin with the following command:

```shell
cargo xtask bundle WahWah --release
```
**Note**: The compiled VST object is located at `target/bundled/WahWah.vst3`. Remember to delete the previous version before recompiling, as it does not overwrite existing files.

## Future Work
- [ ] Preset Management: Users will be able to save and load their settings.
- [ ] Enhanced DAW Integration: Include DAW automation and clock-syncing features.
- [ ] Improved User Interface: Develop a more intuitive and visually appealing UI.
- [ ] Expanded Modulation Options: Add additional LFO shapes and modulation sources.
- [ ] Mobile Compatibility: Adapt the plugin for use in mobile DAW applications.

## Development Guide
The plugin is built using the NIH-Plug framework, which you can explore here: [NIH-Plug framework](https://github.com/robbert-vdh/nih-plug). The core functionality revolves around a dynamic bandpass filter controlled by an LFO and an onset-detection algorithm for automatic modulation.

### Contributing
We highly value contributions and are particularly interested in the following areas, listed in order of importance:
- Preset management implementation
- User interface enhancements
- DAW integration features
- Expanding modulation capabilities
- Adapting the plugin for mobile platforms

Developers are encouraged to contribute to these areas to help enhance the WahWah project. We appreciate your support and collaboration!
