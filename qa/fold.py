import sys
from playwright.sync_api import sync_playwright
with sync_playwright() as p:
    b=p.chromium.launch()
    for w,h in [(390,844),(1920,1040),(1280,800)]:
        pg=b.new_page(viewport={'width':w,'height':h})
        pg.goto('http://127.0.0.1:8787/')
        r=pg.evaluate("""()=>{const q=s=>{const e=document.querySelector(s);return e?Math.round(e.getBoundingClientRect().top):null};
        const c=document.querySelector('.grid .card, .grid > *');const cr=c&&c.getBoundingClientRect();
        return {h1:q('h1'),news:q('.news'),grid:q('.grid'),cardBottom:cr&&Math.round(cr.bottom),header:Math.round(document.querySelector('header').getBoundingClientRect().bottom)}}""")
        print(w,h,r)
        if len(sys.argv)>1: pg.screenshot(path=f'shots/{sys.argv[1]}-{w}.png')
    b.close()
