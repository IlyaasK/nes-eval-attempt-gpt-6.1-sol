import numpy as np,sys,zlib,struct
x=np.fromfile(sys.argv[1],dtype='<u2').reshape(240,256)
pal=np.fromfile('spec/nes_palette.pal',dtype='u1').reshape(-1,3)
y=np.repeat(np.repeat(pal[x&511],2,0),2,1)
def chunk(k,v):return struct.pack('>I',len(v))+k+v+struct.pack('>I',zlib.crc32(k+v))
open(sys.argv[2],'wb').write(b'\x89PNG\r\n\x1a\n'+chunk(b'IHDR',struct.pack('>IIBBBBB',512,480,8,2,0,0,0))+chunk(b'IDAT',zlib.compress(b''.join(b'\0'+row.tobytes() for row in y)))+chunk(b'IEND',b''))
