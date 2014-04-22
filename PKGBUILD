pkgname=systemd-crontab-generator
pkgver=0.1
pkgrel=1
pkgdesc="systemd generator to generate timers/services from crontab and anacrontab files"
url="https://gist.github.com/kstep/4533178982bec6864ab8"
arch=('any')
license=('GPLv3')
depends=('python2' 'systemd')
provides=('cron' 'anacron')
replaces=('cron' 'anacron')
source=(systemd-crontab-generator
        systemd-crontab-update)
md5sum=('78fa326662d0169fc0666b25afc39290'
        'dbff9858b17a171a11c146545c4862da')

build() {
}

package() {
    cd "${srcdir}/${pkgname}-${pkgver}"
    install --owner=root --group=root --mode=0755 systemd-crontab-generator /usr/lib/systemd/system-generators
    install --owner=root --group=root --mode=0755 systemd-crontab-update /usr/bin
}
