#!/usr/bin/python2
import sys
import os
import re

def files(dirname):
    try:
        return filter(os.path.isfile, map(lambda f: os.path.join(dirname, f), os.listdir(dirname)))
    except OSError:
        return []

envvar_re = re.compile(r'^([A-Za-z_0-9]+)\s*=\s*(.*)$')

CRONTAB_FILES = ['/etc/crontab'] + files('/etc/cron.d')
ANACRONTAB_FILES = ['/etc/anacrontab']
USERCRONTAB_FILES = files('/var/spool/cron')

TARGER_DIR = sys.argv[1]
SELF = os.path.basename(sys.argv[0])

def parse_crontab(filename, withuser=True, monotonic=False):
    basename = os.path.basename(filename)
    environment = {}
    with open(filename, 'r') as f:
        for line in f.readlines():
            if line.startswith('#'):
                continue

            line = line.rstrip('\n')
            envvar = envvar_re.match(line)
            if envvar:
                environment[envvar.group(1)] = envvar.group(2)

            if not line or '=' in line:
                continue

            parts = line.split()

            if monotonic:
                period, delay, jobid = parts[0:3]
                command = parts[3:]
                period = {
                        '1': 'daily',
                        '7': 'weekly',
                        '@annually': 'yearly'
                        }.get(period, None) or period.lstrip('@')

                yield {
                        'e': ' '.join('"%s=%s"' % kv for kv in environment.iteritems()),
                        'l': line,
                        'f': filename,
                        'p': period,
                        'd': delay,
                        'j': jobid,
                        'c': ' '.join(command),
                        'u': 'root'
                        }

            else:
                if line.startswith('@'):
                    period = parts[0]
                    period = {
                            '1': 'daily',
                            '7': 'weekly',
                            '@annually': 'yearly'
                            }.get(period, None) or period.lstrip('@')

                    user, command = (parts[1], parts[2:]) if withuser else (basename, parts[1:])

                    yield {
                            'e': ' '.join('"%s=%s"' % kv for kv in environment.iteritems()),
                            'l': line,
                            'f': filename,
                            'p': period,
                            'u': user,
                            'c': ' '.join(command)
                            }
                else:
                    minutes, hours, days = parts[0:3]
                    months, dows = parts[3:5]
                    user, command = (parts[5], parts[6:]) if withuser else (basename, parts[5:])

                    yield {
                            'e': ' '.join('"%s=%s"' % kv for kv in environment.iteritems()),
                            'l': line,
                            'f': filename,
                            'm': parse_time_unit(minutes, range(0, 60)),
                            'h': parse_time_unit(hours, range(0, 24)),
                            'd': parse_time_unit(days, range(0, 32)),
                            'w': parse_time_unit(dows, ['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat'], dow_map),
                            'M': parse_time_unit(months, range(0, 13), month_map),
                            'u': user,
                            'c': ' '.join(command)
                            }

def parse_time_unit(value, values, mapping=int):
    if value == '*':
        return ['*']
    return list(reduce(lambda a, i: a.union(set(i)), map(values.__getitem__,
        map(parse_period(mapping), value.split(','))), set()))

def month_map(month):
    try:
        return int(month)
    except ValueError:
        return ['jan', 'feb', 'mar', 'apr', 'may', 'jun', 'jul', 'aug', 'sep', 'nov', 'dec'].index(month.lower()[0:3]) + 1

def dow_map(dow):
    try:
        return ['sun', 'mon', 'tue', 'wed', 'thu', 'fri', 'sat'].index(dow[0:3].lower())
    except ValueError:
        return int(dow) % 7

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
    unit_name = "cron-%s-%s" % (job['u'], n)
    
    if 'p' in job:
        if job['p'] == 'reboot':
            schedule = 'OnBootSec=5m'
        else:
            try:
                schedule = 'OnCalendar=*-*-1/%s 0:%s:0' % (int(job['p']), job.get('d', 0))
            except ValueError:
                schedule = 'OnCalendar=%s' % job['p']

        accuracy = job.get('d', 1)

    else:
        dows = ','.join(job['w'])
        dows = '' if dows == '*' else dows + ' '

        schedule = 'OnCalendar=%s*-%s-%s %s:%s:00' % (dows, ','.join(map(str, job['M'])),
                ','.join(map(str, job['d'])), ','.join(map(str, job['h'])), ','.join(map(str, job['m'])))
        accuracy = 1

    with open('%s/%s.timer' % (TARGER_DIR, unit_name), 'w') as f:
        f.write('''# Automatically generated by %s
# Source crontab: %s

[Unit]
Description=[Cron] "%s"
PartOf=cron.target
RefuseManualStart=true
RefuseManualStop=true

[Timer]
Unit=%s.service
Persistent=true
AccuracySec=%sm
%s
''' % (SELF, job['f'], job['l'], unit_name, accuracy, schedule))

    with open('%s/%s.service' % (TARGER_DIR, unit_name), 'w') as f:
        f.write('''# Automatically generated by %s
# Source crontab: %s

[Unit]
Description=[Cron] "%s"
RefuseManualStart=true
RefuseManualStop=true

[Service]
Type=oneshot
User=%s
Environment=%s
ExecStart=/bin/sh -c '%s'
''' % (SELF, job['f'], job['l'], job['u'], job['e'], job['c']))

    return '%s.timer' % unit_name

seqs = {}
def count():
    n = 0
    while True:
        yield n
        n += 1

requirements = []

for filename in CRONTAB_FILES:
    try:
        for job in parse_crontab(filename, withuser=True):
            requirements.append(generate_timer_unit(job, seqs.setdefault(job['u'], count())))
    except IOError:
        pass

for filename in ANACRONTAB_FILES:
    try:
        for job in parse_crontab(filename, monotonic=True):
            requirements.append(generate_timer_unit(job, seqs.setdefault(job['u'], count())))
    except IOError:
        pass

for filename in USERCRONTAB_FILES:
    try:
        for job in parse_crontab(filename, withuser=False):
            requirements.append(generate_timer_unit(job, seqs.setdefault(job['u'], count())))
    except IOError:
        pass

with open('%s/cron.target' % (TARGER_DIR), 'w') as f:
    f.write('''# Automatically generated by %s

[Unit]
Description=Cron Jobs
Requires=%s

[Install]
WantedBy=multi-user.target
''' % (SELF, ' '.join(requirements)))
