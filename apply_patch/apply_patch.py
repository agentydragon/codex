#!/usr/bin/env python3
"""
Apply stripped-down patch spec to files.
Spec: codex-rs/core/prompt.md lines 41-74.
"""
import sys
import os

def parse_patch(text):
    lines = text.splitlines()
    if not lines or not lines[0].startswith('*** Begin Patch'):
        sys.exit('Missing *** Begin Patch')
    if not lines[-1].startswith('*** End Patch'):
        sys.exit('Missing *** End Patch')
    ops = []
    i = 1
    end = len(lines) - 1
    while i < end:
        line = lines[i]
        if line.startswith('*** Add File: '):
            path = line[len('*** Add File: '):]
            i += 1
            content = []
            while i < end and lines[i].startswith('+'):
                content.append(lines[i][1:])
                i += 1
            ops.append({'op': 'add', 'path': path, 'content': content})
        elif line.startswith('*** Delete File: '):
            path = line[len('*** Delete File: '):]
            ops.append({'op': 'delete', 'path': path})
            i += 1
        elif line.startswith('*** Update File: '):
            path = line[len('*** Update File: '):]
            i += 1
            move_to = None
            if i < end and lines[i].startswith('*** Move to: '):
                move_to = lines[i][len('*** Move to: '):]
                i += 1
            hunks = []
            while i < end and lines[i].startswith('@@'):
                i += 1
                hunk = []
                while i < end and lines[i] and lines[i][0] in (' ', '-', '+'):
                    hunk.append((lines[i][0], lines[i][1:]))
                    i += 1
                if i < end and lines[i].startswith('*** End of File'):
                    i += 1
                hunks.append(hunk)
            ops.append({'op': 'update', 'path': path, 'move_to': move_to, 'hunks': hunks})
        elif not line.strip():
            i += 1
        else:
            sys.exit(f'Unexpected patch line: {line}')
    return ops

def apply_hunks(old_lines, hunks):
    new_lines = []
    old_index = 0
    for hunk in hunks:
        prefix = []
        for typ, text in hunk:
            if typ == ' ':
                prefix.append(text)
            else:
                break
        if prefix:
            match = None
            for idx in range(old_index, len(old_lines) - len(prefix) + 1):
                if all(old_lines[idx + j].rstrip('\n') == prefix[j] for j in range(len(prefix))):
                    match = idx
                    break
            if match is None:
                sys.exit(f'Hunk context not found: {prefix}')
        else:
            match = old_index
        new_lines.extend(old_lines[old_index:match])
        cur = match
        for typ, text in hunk:
            if typ == ' ':
                new_lines.append(old_lines[cur])
                cur += 1
            elif typ == '-':
                cur += 1
            elif typ == '+':
                new_lines.append(text + '\n')
        old_index = cur
    new_lines.extend(old_lines[old_index:])
    return new_lines

def main():
    patch_text = sys.stdin.read()
    ops = parse_patch(patch_text)
    for op in ops:
        if op['op'] == 'add':
            os.makedirs(os.path.dirname(op['path']) or '.', exist_ok=True)
            with open(op['path'], 'w') as f:
                for line in op['content']:
                    f.write(line + '\n')
        elif op['op'] == 'delete':
            try:
                os.remove(op['path'])
            except FileNotFoundError:
                pass
        elif op['op'] == 'update':
            with open(op['path'], 'r') as f:
                old = f.readlines()
            new = apply_hunks(old, op['hunks'])
            with open(op['path'], 'w') as f:
                f.writelines(new)
            if op['move_to']:
                os.makedirs(os.path.dirname(op['move_to']) or '.', exist_ok=True)
                os.rename(op['path'], op['move_to'])
    print('Done!')

if __name__ == '__main__':
    main()
