# Maintainer: Konstantin Stepanov <me@kstep.me>
pkgname=systemd-crontab-generator
pkgver=0.4
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
        cron.target
        crontab)
md5sums=('3bd87778cb646470108826e9933f48fd'
         '054206bd63854dd6b27890c855a34ce8'
         '97450f27b69a1e88f1b21faad403df7c'
         'a6acfeaae92832bf123b4059744eb0f1')

build() {
    echo
}

package() {
    install --mode=0755 -D systemd-crontab-generator ${pkgdir}/usr/lib/systemd/system-generators/systemd-crontab-generator
    install --mode=0755 -D systemd-crontab-update ${pkgdir}/usr/bin/systemd-crontab-update
    install --mode=0644 -D cron.target ${pkgdir}/usr/lib/systemd/system/cron.target
    install --mode=0755 -D crontab ${pkgdir}/usr/bin/crontab
}
