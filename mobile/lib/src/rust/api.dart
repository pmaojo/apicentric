// This file is a placeholder.
// Run `flutter_rust_bridge_codegen generate` to generate the actual code.

import 'package:flutter_rust_bridge/flutter_rust_bridge_for_generated.dart';

Future<void> initApp() async {}

Future<String> greet({required String name}) async {
  return "Hello $name (Mock)";
}

class ServiceStatus {
  final String name;
  final int port;
  final bool isActive;

  ServiceStatus({required this.name, required this.port, required this.isActive});
}

Future<void> startSimulator({required String configPath, required String dbPath}) async {}
Future<void> stopSimulator() async {}
Future<List<ServiceStatus>> getActiveServices() async { return []; }
Future<void> saveServiceDefinition({required String filename, required String content}) async {}
Future<String> generateServiceAi({required String apiKey, required String prompt, required String model}) async { return ""; }
Future<void> createLogStream({required StreamSink<String> sink}) async {}
