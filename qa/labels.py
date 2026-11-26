from playwright.sync_api import sync_playwright
import sys; B=sys.argv[1] if len(sys.argv)>1 else "http://127.0.0.1:8787"
with sync_playwright() as p:
    b=p.chromium.launch()
    for vp,tag in [((1440,900),'d'),((1024,768),'t'),((390,844),'m')]:
        pg=b.new_page(viewport={'width':vp[0],'height':vp[1]}); pg.goto(B+'/'); pg.evaluate("localStorage.setItem('theme','grey')")
        pg.goto(B+'/fleet/'); pg.locator('#fleet_agents_per_gib').screenshot(path=f'shots/lbl-{tag}.png')
        bad=[]
        for u in ['/','/fleet/','/resources/','/failures/','/hostile/','/tokens/','/governance/','/harness/pi/']:
            pg.goto(B+u)
            bad+=pg.evaluate("""(u)=>{const out=[];for(const svg of document.querySelectorAll('main svg')){if(!svg.getBoundingClientRect().width)continue;const ts=[...svg.querySelectorAll('text')].map(t=>t.getBoundingClientRect());const sr=svg.getBoundingClientRect();for(let i=0;i<ts.length;i++){const a=ts[i];if(a.right>sr.right+1||a.left<sr.left-1)out.push(u+' '+(svg.closest('section')||{}).id+' clipped');for(let j=i+1;j<ts.length;j++){const b=ts[j];if(a.left<b.right-1&&b.left<a.right-1&&a.top<b.bottom-1&&b.top<a.bottom-1)out.push(u+' '+(svg.closest('section')||{}).id)}}}return [...new Set(out)]}""",u)
        print(tag,'problems',bad)
        pg.close()
    b.close()
