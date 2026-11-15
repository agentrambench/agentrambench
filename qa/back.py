from playwright.sync_api import sync_playwright
with sync_playwright() as p:
    b=p.chromium.launch(); pg=b.new_page(viewport={'width':1280,'height':800})
    pg.goto('http://127.0.0.1:8787/method/')
    pg.evaluate('scrollTo(0,1500)'); pg.wait_for_timeout(200); print('y',pg.evaluate('scrollY'), pg.evaluate('document.documentElement.scrollHeight'))
    pg.click('nav.main a[href="/harnesses/"]'); pg.wait_for_timeout(400)
    pg.go_back(); pg.wait_for_timeout(600); print(pg.evaluate('[location.pathname, history.state, scrollY, document.documentElement.scrollHeight]'))
    b.close()
