"""Run the bundled diagnostics natively and retain final frames/audio locally."""
import pathlib,subprocess,os,json
results=[]
for rom in sorted(pathlib.Path('dev-roms/test').glob('*.nes')):
 frames=1800 if rom.name=='instr_timing.nes' else 600
 dest=pathlib.Path('.out/regression')/rom.stem
 p=subprocess.run(['target/release/examples/run',str(rom),str(frames),str(dest)],env=dict(os.environ,LAST_ONLY='1'),capture_output=True,text=True)
 result=dict(rom=rom.name,frames=frames,exit=p.returncode,output=p.stdout.strip(),error=p.stderr.strip());results.append(result)
 print(rom.name, p.stdout.strip().replace('\n',' | '),flush=True)
pathlib.Path('.out/regression/results.json').write_text(json.dumps(results,indent=2))
