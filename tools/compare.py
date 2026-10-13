import argparse,numpy as np,wave,pathlib
p=argparse.ArgumentParser();p.add_argument('ours');p.add_argument('ref');a=p.parse_args()
for f in sorted(pathlib.Path(a.ours).glob('frame_*.idx')):
 r=pathlib.Path(a.ref)/f.name
 if not r.exists():continue
 x=np.fromfile(f,dtype='<u2');y=np.fromfile(r,dtype='<u2');n=np.count_nonzero(x!=y)
 if n: print(f.name,n, 'bbox',[(int(z.min()),int(z.max())) for z in np.where((x!=y).reshape(240,256))])
