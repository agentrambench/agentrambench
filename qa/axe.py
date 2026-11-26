import sys,collections
from playwright.sync_api import sync_playwright
AXE=open('/home/sr/.cache/agentrambench/axe.min.js').read()
B=sys.argv[1]
pages=['/','/fleet/','/resources/','/failures/','/hostile/','/tokens/','/governance/','/harnesses/','/harness/pi/','/method/','/data/','/editions/','/submit/','/about/','/nope']
agg={}
with sync_playwright() as p:
    b=p.chromium.launch()
    for theme in ['light','grey','dark']:
      for vp in [(1440,900),(390,844)]:
        pg=b.new_page(viewport={'width':vp[0],'height':vp[1]})
        pg.goto(B+'/'); pg.evaluate(f"localStorage.setItem('theme','{theme}')")
        for u in pages:
            pg.goto(B+u); pg.add_script_tag(content=AXE)
            res=pg.evaluate("axe.run(document,{runOnly:['wcag2a','wcag2aa','wcag21aa','wcag22aa','best-practice']})")
            for v in res['violations']:
                a=agg.setdefault(v['id'],{'n':0,'pages':set(),'themes':set(),'impact':v['impact'],'help':v['help'],'ex':v['nodes'][0]['target'][0] if v['nodes'] else ''})
                a['n']+=len(v['nodes']); a['pages'].add(u); a['themes'].add(theme)
        pg.close()
    pg=b.new_page(); pg.goto(B+'/')
    seq=[]
    for i in range(5):
        pg.keyboard.press('Tab'); seq.append(pg.evaluate("(()=>{const e=document.activeElement;const s=getComputedStyle(e);return (e.innerText||e.getAttribute('aria-label')||e.tagName).trim().slice(0,18)+' '+s.outlineStyle})()"))
    print('TAB',seq)
    b.close()
for k,v in sorted(agg.items(), key=lambda x:-x[1]['n']):
    print(f"{v['impact']:9} {k:26} n={v['n']:4} themes={sorted(v['themes'])} pages={sorted(v['pages'])[:5]} ex={v['ex']} | {v['help']}")
