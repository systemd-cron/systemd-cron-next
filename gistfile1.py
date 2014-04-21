#!/usr/bin/python2
import sys
import os

def files(dirname):
    return filter(os.path.isfile, map(lambda f: os.path.join(dirname, f), os.listdir(dirname)))

CRONTAB_FILES = ['/etc/crontab'] + files('/etc/cron.d')
ANACRONTAB_FILES = ['/etc/anacrontab']
USERCRONTAB_FILES = files('/var/spool/cron')

TARGER_DIR = sys.argv[1]

def parse_crontab(filename, withuser=True, monotonic=False):
    basename = os.path.basename(filename)
    with open(filename, 'r') as f:
        for line in f.readlines():
            if line.startswith('#'):
                continue

            line = line.strip()
            if not line or '=' in line:
                continue

            parts = line.split()

            if monotonic:
                period, delay, jobid = parts[0:3]
                command = parts[3:]

                yield {
                        'p': period.lstrip('@'),
                        'd': delay,
                        'j': jobid,
                        'c': ' '.join(command),
                        'u': 'root'
                        }

            else:
                if line.startswith('@'):
                    period = parts[0].lstrip('@')
                    user, command = (parts[1], parts[2:]) if withuser else (basename, parts[1:])

                    yield {
                            'p': period,
                            'u': user,
                            'c': ' '.join(command)
                            }
                else:
                    minutes, hours, days = parts[0:3]
                    dows, months = parts[3:5]
                    user, command = (parts[5], parts[6:]) if withuser else (basename, parts[5:])

                    yield {
                            'm': parse_time_unit(minutes, range(0, 60)),
                            'h': parse_time_unit(hours, range(0, 24)),
                            'd': parse_time_unit(days, range(1, 32)),
                            'w': parse_time_unit(dows, ['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat'], dow_map),
                            'M': parse_time_unit(months, range(1, 13), month_map),
                            'u': user,
                            'c': ' '.join(command)
                            }

def parse_time_unit(value, values, mapping=int):
    if value == '*':
        return ['*']
    return list(reduce(lambda a, i: a + set(i), map(values.__getitem__,
        map(parse_period(mapping), value.split(',')))))

def month_map(month):
    month = month.lower()
    try:
        return ['jan', 'feb', 'mar', 'apr', 'may', 'jun', 'jul', 'aug', 'sep', 'nov', 'dec'].index(month.lower()[0:3]) + 1
    except ValueError:
        return int(month)

def dow_map(dow):
    try:
        return ['sun', 'mon', 'tue', 'wed', 'thu', 'fri', 'sat'][int(dow) % 7]
    except ValueError as e:
        return dow[0:3].lower()

def parse_period(mapping=int):
    def parser(value):
        try:
            range, step = value.split('/')
        except ValueError:
            value = mapping(value)
            return slice(value, value + 1)

        if range == '*':
            return slice(None, None, int(step))

        try:
            start, end = range.split('-')
        except ValueError:
            return slice(mapping(range), None, int(step))

        return slice(mapping(start), mapping(end), int(step))

    return parser

def generate_timer_unit(job, seq):
    n = next(seq)
    unit_name = "%s-%s" % (job['u'], n)

    with open('%s/%s.timer' % (TARGER_DIR, unit_name), 'w') as f:
        f.write('[Unit]\n')
        f.write('Description=Crontab entry for "%s"\n' % job['c'])

        f.write('[Timer]\n')
        if 'p' in job:
            if job['p'] == 'reboot':
                f.write('OnBootSec=5\n')
            else:
                f.write('OnCalendar=%s\n' % job['p'])

        else:
            f.write('OnCalendar=%s %s-%s %s:%s\n' % (','.join(job['w']), ','.join(map(str, job['M'])),
                ','.join(map(str, job['d'])), ','.join(map(str, job['h'])), ','.join(map(str, job['m']))))

    with open('%s/%s.service' % (TARGER_DIR, unit_name), 'w') as f:
        f.write('[Unit]\n')
        f.write('Description=Crontab command "%s"\n' % job['c'])
        f.write('[Service]\n')
        f.write('Type=oneshot\n')
        f.write('User=%s\n' % job['u'])
        f.write('ExecStart=%s\n' % job['c'])
        f.write('[Install]\n')
        f.write('WantedBy=crontab.target')

seqs = {}
def count():
    n = 0
    while True:
        yield n
        n += 1

for filename in CRONTAB_FILES:
    try:
        for job in parse_crontab(filename, withuser=True):
            generate_timer_unit(job, seqs.setdefault(job['u'], count()))
    except IOError:
        pass

for filename in ANACRONTAB_FILES:
    try:
        for n, job in enumerate(parse_crontab(filename, monotonic=True)):
            generate_timer_unit(job, seqs.setdefault(job['u'], count()))
    except IOError:
        pass

for filename in USERCRONTAB_FILES:
    try:
        for n, job in enumerate(parse_crontab(filename, withuser=False)):
            generate_timer_unit(job, seqs.setdefault(job['u'], count()))
    except IOError:
        pass
