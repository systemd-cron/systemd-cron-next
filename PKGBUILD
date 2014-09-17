# Maintainer: Konstantin Stepanov <me@kstep.me>
pkgname=systemd-crontab-generator
pkgver=0.8
pkgrel=1
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
        cron-update.path
        cron-update.service
        crontab
        systemd-crontab-generator.8
        crontab.1
        crontab.5
        anacrontab.5)
md5sums=('8427c0f1420f17078f5668bfe811f042'
         '6f00710ad710e319b52edef3e98bd010'
         '3a3f232316b3cc7942844226d35cb833'
         'cd29641a1a6fcef7940a584e375798f7'
         'fe839a0330e54aad21f86299b90842b4'
         '4ac2cfc8de6dabf2e08f39b3c3557879'
         'f4ed527f0b0bd881f77839dd92bac997'
         'd863925d682395cef72701725f180884'
         'f5e92c03bcb37acd580e2e27f5facc6a'
         '78e87a252f4134c5d6dbd2130dc0a8dc')

build() {
    echo
}

package() {
    install --mode=0755 -D systemd-crontab-generator ${pkgdir}/usr/lib/systemd/system-generators/systemd-crontab-generator
    install --mode=0644 -D man/systemd-crontab-generator.8 ${pkgdir}/usr/share/man/man1/systemd-crontab-generator.8
    gzip ${pkgdir}/usr/share/man/man1/systemd-crontab-generator.8
    install --mode=0755 -D systemd-crontab-update ${pkgdir}/usr/bin/systemd-crontab-update
    install --mode=0644 -D cron.target ${pkgdir}/usr/lib/systemd/system/cron.target
    install --mode=0644 -D cron-update.path ${pkgdir}/usr/lib/systemd/system/cron-update.path
    install --mode=0644 -D cron-update.service ${pkgdir}/usr/lib/systemd/system/cron-update.service
    install --mode=0755 -D crontab ${pkgdir}/usr/bin/crontab
    install --mode=0644 -D man/crontab.1 ${pkgdir}/usr/share/man/man1/crontab.1
    gzip ${pkgdir}/usr/share/man/man1/crontab.1
    install --mode=0644 -D man/crontab.5 ${pkgdir}/usr/share/man/man5/crontab.5
    gzip ${pkgdir}/usr/share/man/man5/crontab.5
    install --mode=0644 -D man/anacrontab.5 ${pkgdir}/usr/share/man/man5/anacrontab.5
    gzip ${pkgdir}/usr/share/man/man5/anacrontab.5
}
