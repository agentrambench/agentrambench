import sys,subprocess,numpy as np
f=sys.argv[1]; W,H=1280,900
raw=subprocess.run(['ffmpeg','-loglevel','error','-i',f,'-f','rawvideo','-pix_fmt','gray','-'],capture_output=True).stdout
n=len(raw)//(W*H); a=np.frombuffer(raw[:n*W*H],np.uint8).reshape(n,H,W)
top=a[:,:80].mean(axis=(1,2)); page=a[:,150:].mean(axis=(1,2))
print(n,'frames')
print('toolbar',' '.join(str(int(x)) for x in top[::3]))
print('page   ',' '.join(str(int(x)) for x in page[::3]))
d=np.abs(np.diff(page)); print('page min/max',int(page.min()),int(page.max()),'biggest jumps',sorted(np.round(d,1))[-5:])
