def runs = ['14.04':'collectd-54',
            '16.04':'collectd-55',
            '17.10':'collectd-57']

def steps = runs.collectEntries {
    ["ubuntu $it.key": job(it.key, it.value)]
}

properties([
    pipelineTriggers([
        cron('@weekly')
    ])
])

parallel steps

def job(os, collectd) {
    return {
        docker.image("ubuntu:${os}").inside {
            checkout scm
            sh "VERSION=${collectd} ci/full.sh"
            junit 'TestResults-*'
        }
    }
}
