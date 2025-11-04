name := "example-sbt-project"

version := "1.0.0"

scalaVersion := "2.13.12"

// Dependencies
libraryDependencies ++= Seq(
  "org.scala-lang" % "scala-library" % scalaVersion.value,
  "org.slf4j" % "slf4j-api" % "1.7.36",
  "ch.qos.logback" % "logback-classic" % "1.2.11",
  
  // Test dependencies
  "org.scalatest" %% "scalatest" % "3.2.15" % Test
)

// Compiler options
scalacOptions ++= Seq(
  "-encoding", "UTF-8",
  "-deprecation",
  "-feature",
  "-unchecked"
)

// JAR packaging
assembly / assemblyJarName := "example-sbt-project.jar"
assembly / mainClass := Some("com.example.Main")
