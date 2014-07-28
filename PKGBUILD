# Maintainer: Konstantin Stepanov <me@kstep.me>
pkgname=systemd-crontab-generator
pkgver=0.7
pkgrel=2
pkgdesc="systemd generator to generate timers/services from crontab and anacrontab files"
url="https://github.com/kstep/systemd-crontab-generator"
arch=('any')
license=('GPL3')
depends=('python2' 'systemd')
provides=('cron' 'anacron')
replaces=('cron' 'anacron')
source=(systemd-crontab-generator
        systemd-crontab-update
        cron.target
        crontab
        systemd-crontab-generator.1
        crontab.1
        crontab.5)
md5sums=('e90e20d2b5f7f6c44bbc10ac821d9532'
         '6f00710ad710e319b52edef3e98bd010'
         '97450f27b69a1e88f1b21faad403df7c'
         'fa6c2b06e1105787ccb10bfce95c18a0'
         '99411ac19ecdac19c1aa6561ce7beb3d'
         '39097a37b4aa687f502ef91994325389'
         'bfffd32858a1aa1a4f5b20e565c010a9')

build() {
    echo
}

package() {
    install --mode=0755 -D systemd-crontab-generator ${pkgdir}/usr/lib/systemd/system-generators/systemd-crontab-generator
    install --mode=0644 -D man/systemd-crontab-generator.1 ${pkgdir}/usr/share/man/man1/systemd-crontab-generator.1
    gzip ${pkgdir}/usr/share/man/man1/systemd-crontab-generator.1
    install --mode=0755 -D systemd-crontab-update ${pkgdir}/usr/bin/systemd-crontab-update
    install --mode=0644 -D cron.target ${pkgdir}/usr/lib/systemd/system/cron.target
    install --mode=0755 -D crontab ${pkgdir}/usr/bin/crontab
    install --mode=0644 -D man/crontab.1 ${pkgdir}/usr/share/man/man1/crontab.1
    gzip ${pkgdir}/usr/share/man/man1/crontab.1
    install --mode=0644 -D man/crontab.5 ${pkgdir}/usr/share/man/man5/crontab.5
    gzip ${pkgdir}/usr/share/man/man5/crontab.5
}
