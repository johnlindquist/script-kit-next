#!/usr/bin/env python3
"""Grade the p3-form-edit chapter's actual glyph, insertion and selection pixels (stock dark theme)."""
import json, sys
from pathlib import Path
from PIL import Image


def read_field(frames, name):
    [file] = frames.glob(f'[0-9][0-9]-{name}.png')
    img = Image.open(file).convert('RGB'); w, h = img.size
    # Find the active field by its accent outline, rather than assuming a field y or height.
    accent = (251,191,36); px = img.load()
    edges = [y for y in range(h) if all(px[int(w*r), y] == accent for r in (.23,.45,.72))]
    assert edges, f'{name}: no active border'
    field = img.crop((int(w*.03), min(edges)+6, int(w*.25), max(edges)-6))
    def bounds(pred):
        mask = Image.new('1',field.size);mask.putdata([int(pred(r,g,b)) for r,g,b in field.getdata()]);return mask.getbbox()
    caret=bounds(lambda r,g,b:(r,g,b)==accent)
    highlight=bounds(lambda r,g,b: r>g+8 and g>b+12 and 45<r<210)
    ink=bounds(lambda r,g,b:min(r,g,b)>190 and max(r,g,b)-min(r,g,b)<16)
    return dict(caret=caret,selection=highlight,ink=ink,top=min(edges))


def grade(frames):
    names=('first','blank','second','selected-word','selected-both','replaced','all','empty','select-email','mixed','middle','insert-middle')
    m={n:read_field(frames,n) for n in names}; a=m['first']['caret']; blank=m['blank']['caret']; c=m['second']['caret']; text=m['second']['ink']
    checks={'newline advances caret one row':blank and a and blank[1]>a[3] and blank[0]<a[0],
            'caret follows ok on second baseline':c and text and blank and c[1]==blank[1] and c[1]<=text[3]<=c[3]+2 and 0<=c[0]-text[2]<=6,
            'selection does not reflow letters':m['selected-word']['ink']==text==m['selected-both']['ink'],
            'only the selected range widens':all(m[n]['selection'] and not m[n]['caret'] for n in ('selected-word','selected-both','all','select-email')) and m['selected-both']['selection'][0]<m['selected-word']['selection'][0],
            'replacing selection restores a collapsed caret':m['replaced']['caret'] and not m['replaced']['selection'] and m['empty']['caret'] and not m['empty']['ink'],
            'moving a caret in proportional text does not move text':m['mixed']['ink']==m['middle']['ink'] and m['middle']['caret'][0]<m['mixed']['caret'][0],
            'Tab transfers the active border':m['select-email']['top']<m['second']['top']}
    checks={k:bool(v) for k,v in checks.items()}; print(json.dumps({'pass':all(checks.values()),'checks':checks,'pixels':m},indent=2));return all(checks.values())

if __name__=='__main__': sys.exit(0 if grade(Path(sys.argv[1])) else 1)
