#!/bin/env python3

import re
import itertools
import collections
from datetime import datetime, timedelta, timezone


PERIOD_DELTA = timedelta(minutes=5)
FAULTY_CONNS_LIMIT = 5


def parse_line(line):
    """ Parse a line in nginx-logfile """
    nginx_default_pattern = (
        r"^(.*?)\s-\s-\s\[(.*?)\]\s\"(\w*)\s(/.*?)\s(.*?)\"\s(\d*?)\s"
        r"(\d*?)\s\"(.*?)\"\s\"(.*?)\"$"
        )
    nginx_default_timestamp = '%d/%b/%Y:%H:%M:%S %z'
    pattern = re.compile(nginx_default_pattern)
    m = re.match(pattern, line)
    if m:
        match = {
            'ip': m.group(1),
            'ts': datetime.strptime(m.group(2),
                                    nginx_default_timestamp),
            'action': m.group(3),
            'path': m.group(4),
            'prot': m.group(5),
            'status': m.group(6),
            'resp_size': m.group(7),
            'referrer': m.group(8),
            'client': m.group(9),
        }
    else:
        match = None
    return match

def parse_logfile(fname, maxlines=None, verbose=False):
    """ Parse access-attemps in nginx-logfile.

    Args:
        fname (str):    Path to nginx-logfile.
        maxlines (str): Max number of lines to parse.
    Yields:
        event (dict):   Yields dict describing event.
    """
    num_lines = 0
    num_failed = 0
    with open(fname, 'r') as f:
        for i, line in enumerate(f):
            if maxlines and i > maxlines:
                print('Reached maximum of {} lines, stopping parse'
                      ''.format(i-1))
                break
            match = parse_line(line)
            if match:
                yield match
            else:
                if verbose:
                    print('No match on line {}:\n{}'.format(i, line))
                num_failed += 1
            num_lines = i
    print('Parsing failed to parse {}/{} lines ({:.2%})'
          ''.format(num_failed, num_lines, num_failed / num_lines))


def events_stats(events):
    """ Helper-function to calculate statistics from event-stream """
    ip_data = collections.defaultdict(lambda:{
        'last_hour': datetime(1977, 1, 1, tzinfo=timezone.utc),
        'succ_conns': 0,
        'faulty_conns': 0,
        'succ_conns_lst_hr': 0,
        'faulty_conns_lst_hr': 0,
        'max_conns_per_hour': 0,
        'max_faulty_conns_per_hour': 0,
    })

    for evt in events:
        d = ip_data[evt['ip']]
        if evt['ts'] > d['last_hour'] + PERIOD_DELTA:
            d['last_hour'] = evt['ts']

            # Record max-numbers during last hour
            if d['succ_conns_lst_hr'] > d['max_conns_per_hour']:
                d['max_conns_per_hour'] = d['succ_conns_lst_hr']
            d['succ_conns_lst_hr'] = 0
            if d['faulty_conns_lst_hr'] > d['max_faulty_conns_per_hour']:
                d['max_faulty_conns_per_hour'] = d['faulty_conns_lst_hr']
            d['faulty_conns_lst_hr'] = 0

        # Record faulty connection attempts (anything else than
        # HTTP-code (2xx)
        # print('status', repr(evt['status']))
        if evt['status'][0] != '2':
            d['faulty_conns'] += 1
            d['faulty_conns_lst_hr'] += 1
        else:
            d['succ_conns'] += 1
            d['succ_conns_lst_hr'] += 1

    return ip_data

def _analyze_most_frequent(ip_data):

    for feat, desc in (('max_conns_per_hour', 'successful'),
                       ('max_faulty_conns_per_hour', 'non-successful')):
        conn_freqs = collections.Counter({k:v[feat]
                                          for k, v in ip_data.items()})
        print('\nMost frequent {} responses\tCount per period'
              .format(desc))
        for i, (ip, freq) in enumerate(conn_freqs.most_common(10)):
            print('#{}         {}\t{}'.format(i+1, ip, freq))


def analyze(logfile, filterfile, maxlines=None, verbose=False):
    """ Main entrypoint for analyzing a logfile and producing filterfile.

    Filterfile can later be used in botdetector reverse-proxy to
    filter out bad actors (scrapers, bots, etc).

    Args:
        logfile (str):      Path to nginx-logfile.
        filterfile (str):   Path to produced filterfile.
        verbose (bool):     Verbose analysis
    """
    events = parse_logfile(logfile, maxlines=maxlines, verbose=verbose)
    ip_data = events_stats(events)

    def _get_max_feature(featname):
        ip, d = max(ip_data.items(),
                    key=lambda d: d[1][featname])
        return ip, d[featname], d

    most_conns = _get_max_feature('succ_conns')
    most_failed_conns = _get_max_feature('faulty_conns')
    most_conns_pr_hr = _get_max_feature('max_conns_per_hour')
    most_failed_conns_pr_hr = _get_max_feature(
        'max_faulty_conns_per_hour')

    print('\nAnalysis of {}. \nPeriod is {}:'
          .format(logfile, PERIOD_DELTA))
    print('Number of ip\'s connected:           {}'
          .format(len(ip_data)))
    print('Most successful connections:          {}\t{}'
          .format(most_conns[0], most_conns[1]))
    print('Most failed connections:              {}\t{}'
          .format(most_failed_conns[0], most_failed_conns[1]))
    print('Most successful connections per period: {}\t{}'
          .format(most_conns_pr_hr[0], most_conns_pr_hr[1]))
    print('Most failed connections per period:     {}\t{}'
          .format(most_failed_conns[0], most_failed_conns[1]))

    _analyze_most_frequent(ip_data)

    def _maybe_blacklist(d):
        return d[1]['max_faulty_conns_per_hour'] > FAULTY_CONNS_LIMIT
    maybe_blacklist = filter(_maybe_blacklist, ip_data.items())
    print('Suggesting blacklisting of following ip\'s')
    for ip, d in maybe_blacklist:
        print(ip)


def parse_args():
    import argparse
    parser = argparse.ArgumentParser(
        description='Train botdetector-filter on nginx-logs.')
    parser.add_argument('LOGFILE',
                        help='Existing nginx-logfile to analyze.')
    parser.add_argument('FILTERFILE',
                        default='.filterfile',
                        help='Where to store produced filter')
    parser.add_argument('-v', '--verbose',
                        action='store_true',
                        default=False,
                        help='Be verbose')
    parser.add_argument('--maxlines', '-m',
                        type=int,
                        default=None,
                        help=('Only parse this many lines from log'))
    return parser.parse_args()


if __name__ == '__main__':
    args = parse_args()

    analyze(args.LOGFILE,
            filterfile=args.FILTERFILE,
            maxlines=args.maxlines,
            verbose=args.verbose)



