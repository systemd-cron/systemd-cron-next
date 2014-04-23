pkgname=systemd-crontab-generator
pkgver=0.2
pkgrel=1
pkgdesc="systemd generator to generate timers/services from crontab and anacrontab files"
url="https://github.com/kstep/systemd-crontab-generator"
arch=('any')
license=('GPLv3')
depends=('python2' 'systemd')
provides=('cron' 'anacron')
replaces=('cron' 'anacron')
source=(systemd-crontab-generator
        systemd-crontab-update
        cron.target)
md5sums=('7875d3b860a67cac1226639d0d8c1763'
         '5b4fdfaf966bc5b16d399a45d2763b11'
         '97450f27b69a1e88f1b21faad403df7c')

build() {
    echo
}

package() {
    install --mode=0755 -D systemd-crontab-generator ${pkgdir}/usr/lib/systemd/system-generators/systemd-crontab-generator
    install --mode=0755 -D systemd-crontab-update ${pkgdir}/usr/bin/systemd-crontab-update
    install --mode=0644 -D cron.target ${pkgdir}/usr/lib/systemd/system/cron.target
}
