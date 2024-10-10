package initializers

import (
	"fmt"
	"os"
	"time"

	"go.uber.org/zap"
	"go.uber.org/zap/zapcore"
)

func InitLogger() {
	createLogDirectory()
	writerSync := getLogWriter()
	encoder := getEncoder()

	core := zapcore.NewTee(
		zapcore.NewCore(encoder, writerSync, zapcore.DebugLevel),
		//zapcore.NewCore(encoder, zapcore.AddSync(os.Stdout), zapcore.DebugLevel),
	)
	logg := zap.New(core, zap.AddCaller())

	zap.ReplaceGlobals(logg)
}

func createLogDirectory() {
	path, _ := os.Getwd()

	if _, err := os.Stat(fmt.Sprintf("%s\\logs", path)); os.IsNotExist(err) {
		_ = os.Mkdir("logs", os.ModePerm)
	}
}

func getLogWriter() zapcore.WriteSyncer {
	path, err := os.Getwd()
	if err != nil {
		panic(err)
	}

	file, err := os.OpenFile(path+"\\logs\\depmap_log.txt", os.O_APPEND|os.O_CREATE|os.O_WRONLY, 0644)
	if err != nil {
		panic(err)
	}

	return zapcore.AddSync(file)
}

func getEncoder() zapcore.Encoder {
	encoderConfig := zap.NewProductionEncoderConfig()
	encoderConfig.EncodeTime = zapcore.TimeEncoder(func(t time.Time, enc zapcore.PrimitiveArrayEncoder) {
		enc.AppendString(t.UTC().Format("2006-01-02T15:04:05z0700"))
	})

	encoderConfig.EncodeLevel = zapcore.CapitalColorLevelEncoder
	return zapcore.NewConsoleEncoder(encoderConfig)
}
