package common

import (
	"os"

	"github.com/sirupsen/logrus"
)

type Logger interface {
	Debugf(format string, args ...any)
	Infof(format string, args ...any)
	Warnf(format string, args ...any)
	Errorf(format string, args ...any)
}

type DefaultLogger struct {
	logrus *logrus.Logger
}

func NewDefaultLogger() *DefaultLogger {
	l := logrus.New()
	l.SetFormatter(&logrus.TextFormatter{
		FullTimestamp: true,
	})

	if _, devMode := os.LookupEnv("LCSM_DEVELOPMENT"); devMode {
		l.SetLevel(logrus.DebugLevel)
	}

	return &DefaultLogger{logrus: l}
}

func (l *DefaultLogger) Debugf(format string, args ...any) {
	l.logrus.Debugf(format, args...)
}

func (l *DefaultLogger) Infof(format string, args ...any) {
	l.logrus.Infof(format, args...)
}

func (l *DefaultLogger) Warnf(format string, args ...any) {
	l.logrus.Warnf(format, args...)
}

func (l *DefaultLogger) Errorf(format string, args ...any) {
	l.logrus.Errorf(format, args...)
}
