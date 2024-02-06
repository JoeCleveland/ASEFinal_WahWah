# ASEFinal_WahWah

## Motivation
This project is a VST compatible plugin emulating a Wah-Wah pedal, such as the classic Cry-Baby.
Unlike a traditional wah-wah effect which requires constant user input, this plugin automates the control over the wah-wah effect for greater ease of use.

## Applications
Our primary user base will be guitar players, but it should be useful for processing piano or any other instrument in a live real-time setting or in the studio.
The product will be VST compatible and hosted within a DAW.

## Functionality
In addition to the basic wah-wah effect our product will have the following features:
  * The pedal control will be automated to align with the onsets of the user's playing
  * The pedal control can also be automated with a LFO either free-running or synced to the DAW tempo
  * The plugin will be compatible with the automation feature of most DAWs
  * The user will be able adjust the width and other parameters of the internal filters
  * Users will be able to save and load presets

## Implementation

The VST will be implemented within the vst-rs framework: [https://github.com/RustAudio/vst-rs]
The core of the audio processing system will be a bandpass filter, or potentially mode variable filter.
The cutoff of this filter will be controllable by an LFO module, and also an onset-detection algorithm for automatic wah-wah playing.
We may experiment with additional post-processing steps such as compression or distortion to give a unique tone color to our plugin.

![plot](https://github.com/JoeCleveland/ASEFinal_WahWah/main/flowhchart.jpg)



