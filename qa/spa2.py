from playwright.sync_api import sync_playwright
B='http://127.0.0.1:8787'
with sync_playwright() as p:
    b=p.chromium.launch(); pg=b.new_page(viewport={'width':1280,'height':800})
    errs=[]; pg.on('pageerror',lambda e:errs.append(str(e))); pg.on('console',lambda m: m.type=='error' and errs.append(m.text))
    pg.goto(B+'/method/'); pg.evaluate('window.__k=1')
    S='[location.pathname+location.hash, !!window.__k, Math.round(scrollY), document.activeElement.tagName, document.querySelector("[aria-live]").textContent]'
    pg.evaluate('scrollTo(0,1500)'); pg.wait_for_timeout(300)
    pg.click('nav.main a[href="/harnesses/"]'); pg.wait_for_timeout(400); print('to harnesses',pg.evaluate(S))
    pg.evaluate('scrollTo(0,200)'); pg.wait_for_timeout(300); print('harnesses max y', pg.evaluate('scrollY'))
    pg.go_back(); pg.wait_for_timeout(500); print('back→method y1500',pg.evaluate(S))
    pg.go_forward(); pg.wait_for_timeout(500); print('fwd→harnesses y200',pg.evaluate(S))
    # race: two clicks
    pg.evaluate('''()=>{const a=document.querySelector('nav.main a[href="/method/"]'),b=document.querySelector('nav.main a[href="/about/"]');a.click();b.click()}'''); pg.wait_for_timeout(800); print('race→about',pg.evaluate(S), pg.evaluate('document.querySelector("h1").textContent'))
    # same-page anchors
    pg.goto(B+'/failures/'); pg.evaluate('window.__k=1')
    pg.evaluate('''()=>{const a=document.createElement('a');a.href='#faults';a.id='tmpa';a.textContent='go';document.querySelector('main').prepend(a)}'''); pg.click('#tmpa'); pg.wait_for_timeout(300)
    y1=pg.evaluate('scrollY'); pg.go_back(); pg.wait_for_timeout(400); print('anchor back (y was',y1,')',pg.evaluate(S), 'tmpa still there:',pg.evaluate('!!document.getElementById("tmpa")'))
    # redirect /about → ?
    pg.goto(B+'/'); pg.evaluate('window.__k=1')
    pg.evaluate('''()=>{const a=document.createElement('a');a.href='/about';a.id='r';a.textContent='ab';document.querySelector('main').prepend(a)}'''); pg.click('#r'); pg.wait_for_timeout(500); print('no-slash link',pg.evaluate(S))
    print('errors',errs); b.close()
