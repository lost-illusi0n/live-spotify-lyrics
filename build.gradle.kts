plugins {
    kotlin("jvm") version "1.5.10"
    id("application")
    id("org.openjfx.javafxplugin") version "0.0.10"
    id("edu.sc.seis.launch4j") version "2.5.0"
}

repositories {
    mavenCentral()
}

launch4j {
    mainClassName = "MainKt"
    outputDir = "app"
    jreMinVersion = "11"
}

dependencies {
    implementation(kotlin("stdlib"))
    implementation("org.jetbrains.kotlinx:kotlinx-coroutines-core:1.5.0")
    implementation("org.jetbrains.kotlinx:kotlinx-coroutines-javafx:1.5.0")
    implementation("org.jsoup:jsoup:1.13.1")
    implementation("no.tornado:tornadofx:1.7.20")
    implementation("org.jire.kotmem:Kotmem:0.86")
}

javafx {
    version = "11"
    modules("javafx.controls", "javafx.graphics")
}

tasks.withType<org.jetbrains.kotlin.gradle.tasks.KotlinCompile> {
    kotlinOptions.jvmTarget = "11"
}