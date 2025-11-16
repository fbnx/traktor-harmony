# traktor-harmony
Automatically generate harmonic playlists from Traktor collection or harmonize existing Traktor playlists by rearranging the track order according to the rules of the camelot wheel:

<p align="center"><img src="https://mixedinkey.com/wp-content/uploads/2020/04/Camelot-Wheel-Mixed-In-Key-Harmonic-Mixing.png" width=500></p>

The tracks in the collection must be analyzed by Traktor - only tracks with available key and bpm metadata are considered and the transition scores are calculated based on the difference in bpm and musical keys according the chart:

<p align="center"><img src="https://mixedinkey.com/wp-content/uploads/2025/04/Camelot-Wheel-Chart.png" width=500></p>

## Requirements

```
pip install xmltodict
```

## Usage
```
usage: harmonize.py [-h] [--output OUTPUT] [--analyze] [--harmonize] [--generate] [--next NEXT] [--length LENGTH] [--start START] [--key KEY] [--bpm BPM] path

harmonize

positional arguments:
  path             Path to collection/playlist NML file

options:
  -h, --help       show this help message and exit
  --output OUTPUT  Output path to store result
  --analyze        Analyze playlist and print average transition score
  --harmonize      Harmonize playlist and output NML file
  --generate       Generate playlist from collection with given length and output M3U file
  --next NEXT      Print matching next track for given track
  --length LENGTH  Length of generated playlist
  --start START    Filename of start track
  --key KEY        Start playlist with given key
  --bpm BPM        Start playlist with given bpm
```

## Examples

### Generate

Generate new playlist from Traktor collection string with given length and start track

```
./harmonize.py --generate --output generated-playlist.m3u --length 20 --start "some music file from collection.flac" ~/Documents/Native\ Instruments/Traktor\ 4.0.0/collection.nml
```

Generate new playlist from Traktor collection string with given length, key, and bpm

```
./harmonize.py --generate --output generated-playlist.m3u --length 20 --key 11m --bpm 128 ~/Documents/Native\ Instruments/Traktor\ 4.0.0/collection.nml
```

### Harmonize

Harmonize existing Traktor playlist

```
./harmonize.py --harmonize --output harmonized-playlist.nml playlist.nml
```

## Next

Print best matching tracks for a given track from Traktor collection

```
./harmonize.py --next "some music file from collection.flac" ~/Documents/Native\ Instruments/Traktor\ 4.0.0/collection.nml
```

### Analyze

Print average transition score of Traktor playlist

```
./harmonize.py --analyze playlist.nml
```