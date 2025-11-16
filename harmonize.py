#!/usr/bin/env python3

import argparse
import xmltodict

from random import choice

keys = {
    0: '1d',
    1: '8d',
    2: '3d',
    3: '10d',
    4: '5d',
    5: '12d',
    6: '7d',
    7: '2d',
    8: '9d',
    9: '4d',
    10: '11d',
    11: '6d',
    12: '10m',
    13: '5m',
    14: '12m',
    15: '7m',
    16: '2m',
    17: '9m',
    18: '4m',
    19: '11m',
    20: '6m',
    21: '1m',
    22: '8m',
    23: '3m',
    }

PERFECT_MIX = 1.0
ADJACENT_MIX = 0.9
SCALE_CHANGE = 0.9
ENERGY_BOOST = 0.8
DIAGONAL_MIX = 0.8
JAWS_MIX = 0.5
MOOD_SHIFTER = 0.5

wheel = {}
for i in range(1, 13):
    wheel[str(i)+'m'] = {
        str(i)+'m': PERFECT_MIX,
        str(i-1 if i > 1 else 12)+'m': ADJACENT_MIX,
        str(i+1 if i < 12 else 1)+'m': ADJACENT_MIX,
        str(i)+'d': SCALE_CHANGE,
        str(i+2 if i < 11 else i-10)+'m': ENERGY_BOOST,
        str(i-1 if i > 1 else 12)+'d': DIAGONAL_MIX,
        str(i+7 if i < 6 else i-5)+'m': JAWS_MIX,
        str(i+3 if i < 10 else i-9)+'d': MOOD_SHIFTER,
    }

    wheel[str(i)+'d'] = {
        str(i)+'d': PERFECT_MIX,
        str(i-1 if i > 1 else 12)+'d': ADJACENT_MIX,
        str(i+1 if i < 12 else 1)+'d': ADJACENT_MIX,
        str(i)+'m': SCALE_CHANGE,
        str(i+2 if i < 11 else i-10)+'d': ENERGY_BOOST,
        str(i+1 if i < 12 else 1)+'m': DIAGONAL_MIX,
        str(i+7 if i < 6 else i-5)+'d': JAWS_MIX,
        str(i-3 if i > 3 else i+9)+'m': MOOD_SHIFTER,
    }

data = {}
collection = {}
paths = {}

def read_collection(path):
    global data
    with open(path) as f:
        data = xmltodict.parse(f.read())

    for track in data['NML']['COLLECTION']['ENTRY']:
        # Only local files and analyzed tracks with key and bpm
        if '@FILE' in track['LOCATION'] and 'MUSICAL_KEY' in track:
            collection[track['LOCATION']['@FILE']] = (keys[int(track['MUSICAL_KEY']['@VALUE'])], round(float(track['TEMPO']['@BPM'])))
            paths[track['LOCATION']['@FILE']] = '/Volumes/' + track['LOCATION']['@VOLUME'] + track['LOCATION']['@DIR'].replace(':', '') + track['LOCATION']['@FILE']

def calculate_transition_score(first, second):
    key1, bpm1 = collection[first]
    key2, bpm2 = collection[second]

    if key1 == key2 and bpm1 == bpm2:
        return 1.0

    key_value = wheel.get(key1).get(key2, 0)

    bpm_diff = abs(bpm1 - bpm2)
    bpm_value = 0.01 * bpm_diff ** 2

    return key_value - bpm_value

def generate_playlist_from_collection(start, length):
    playlist = [start]
    for i in range(length-1):
        best = []
        score = -999
        for filename in collection.keys():
            if filename in playlist:
                continue
            res = calculate_transition_score(playlist[-1], filename)
            if res > score:
                score = res
                best = [filename]
            elif res == score:
                best.append(filename)

        print(f'{playlist[-1]} -> {best}: {score}')
        playlist.append(choice(best))

    return playlist

def generate_playlist_with_start_track(start, length, output):
    playlist = generate_playlist_from_collection(start, length)
    with open(output, 'w') as f:
        for track in playlist:
            f.write(paths[track] + '\r\n')

def generate_playlist_with_key_and_bpm(key, bpm, length, output):
    start = choice([k for k, v in collection.items() if v[0] == key and v[1] == bpm])
    generate_playlist_with_start_track(start, length, output)

def get_next_track(track):
    print(generate_playlist_from_collection(track, 2))

def get_filename(item):
    return item['PRIMARYKEY']['@KEY'].split(':')[-1]

def analyze_playlist():
    playlist = data['NML']['PLAYLISTS']['NODE']['SUBNODES']['NODE']['PLAYLIST']['ENTRY']
    scores = []
    for i in range(1, len(playlist)):
        prev = get_filename(playlist[i-1])
        cur = get_filename(playlist[i])
        scores.append(calculate_transition_score(prev, cur))
    print(f'Average score: {sum(scores)/float(len(scores))}')

def harmonize_playlist(output):
    playlist = data['NML']['PLAYLISTS']['NODE']['SUBNODES']['NODE']['PLAYLIST']['ENTRY']
    harmonized = generate_playlist_from_collection(get_filename(playlist[0]), len(playlist))
    for i in range(len(harmonized)):
        for j in range(i, len(playlist)):
            if harmonized[i] == get_filename(playlist[j]):
                # swap tracks and put best track at current position
                tmp = playlist[i]
                playlist[i] = playlist[j]
                playlist[j] = tmp

    data['NML']['PLAYLISTS']['NODE']['SUBNODES']['NODE']['PLAYLIST']['ENTRY'] = playlist

    with open(output, 'w') as f:
        f.write(xmltodict.unparse(data))

def main(args):
    read_collection(args.path)

    if args.next:
        get_next_track(args.next)
    elif args.analyze:
        analyze_playlist()
    else:
        if not args.output:
            print('Output path is missing!')
            return

        if args.harmonize:
            harmonize_playlist(args.output)
            return

        if not args.length:
            print('Length is missing!')
            return

        if args.generate:
            if not args.start and not (args.key and args.bpm):
                print('You have to either provide start track or key and bpm!')
            if args.start:
                generate_playlist_with_start_track(args.start, args.length, args.output)
            elif args.key and args.bpm:
                generate_playlist_with_key_and_bpm(args.key, args.bpm, args.length, args.output)

if __name__ == '__main__':
    parser = argparse.ArgumentParser(description='harmonize')
    parser.add_argument('path', type=str, help='Path to collection/playlist NML file')
    parser.add_argument('--output', type=str, help='Output path to store result')
    parser.add_argument('--analyze', default=False, action='store_true', help='Analyze playlist and print average transition score')
    parser.add_argument('--harmonize', default=False, action='store_true', help='Harmonize playlist and output NML file')
    parser.add_argument('--generate', default=False, action='store_true', help='Generate playlist from collection with given length and output M3U file')
    parser.add_argument('--next', type=str, default=None, help='Print matching next track for given track')
    parser.add_argument(
        '--length',
        type=int,
        default=None,
        help='Length of generated playlist',
    )
    parser.add_argument(
        '--start',
        type=str,
        default=None,
        help='Filename of start track',
    )
    parser.add_argument(
        '--key',
        type=str,
        default=None,
        help='Start playlist with given key',
    )
    parser.add_argument(
        '--bpm',
        type=int,
        default=None,
        help='Start playlist with given bpm',
    )

    main(parser.parse_args())