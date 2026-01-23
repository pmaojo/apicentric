import 'package:flutter/material.dart';
import 'package:path_provider/path_provider.dart';
import '../src/rust/api.dart' as rust_api;
import 'dart:io';

class CreateScreen extends StatelessWidget {
  const CreateScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return DefaultTabController(
      length: 2,
      child: Scaffold(
        appBar: AppBar(
          title: const Text("Create Simulation"),
          bottom: const TabBar(
            tabs: [
              Tab(text: "Manual"),
              Tab(text: "AI Wizard"),
            ],
          ),
        ),
        body: const TabBarView(
          children: [
            ManualWizard(),
            AiWizard(),
          ],
        ),
      ),
    );
  }
}

class ManualWizard extends StatefulWidget {
  const ManualWizard({super.key});
  @override
  State<ManualWizard> createState() => _ManualWizardState();
}

class _ManualWizardState extends State<ManualWizard> {
  final _formKey = GlobalKey<FormState>();
  final _nameController = TextEditingController();
  final _portController = TextEditingController(text: "9000");

  Future<void> _save() async {
    if (_formKey.currentState!.validate()) {
        final yaml = """
name: ${_nameController.text}
version: "1.0"
server:
  port: ${_portController.text}
  base_path: /api

endpoints:
  - method: GET
    path: /hello
    responses:
      200:
        body: "Hello from ${_nameController.text}"
""";
        final dir = await getApplicationDocumentsDirectory();
        final filename = "${dir.path}/${_nameController.text.replaceAll(' ', '_').toLowerCase()}.yaml";
        await File(filename).writeAsString(yaml);
        if (mounted) {
            ScaffoldMessenger.of(context).showSnackBar(SnackBar(content: Text("Saved to $filename")));
        }
    }
  }

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.all(16.0),
      child: Form(
        key: _formKey,
        child: Column(
          children: [
            TextFormField(
              controller: _nameController,
              decoration: const InputDecoration(labelText: "Service Name"),
              validator: (v) => v!.isEmpty ? "Required" : null,
            ),
            TextFormField(
              controller: _portController,
              decoration: const InputDecoration(labelText: "Port"),
              keyboardType: TextInputType.number,
            ),
            const SizedBox(height: 20),
            ElevatedButton(onPressed: _save, child: const Text("Create Service"))
          ],
        ),
      ),
    );
  }
}

class AiWizard extends StatefulWidget {
  const AiWizard({super.key});
  @override
  State<AiWizard> createState() => _AiWizardState();
}

class _AiWizardState extends State<AiWizard> {
  final _promptController = TextEditingController();
  final _apiKeyController = TextEditingController();
  bool _loading = false;

  Future<void> _generate() async {
      if (_apiKeyController.text.isEmpty) {
          ScaffoldMessenger.of(context).showSnackBar(const SnackBar(content: Text("API Key is required")));
          return;
      }
      setState(() => _loading = true);
      try {
          final yaml = await rust_api.generateServiceAi(
              apiKey: _apiKeyController.text,
              prompt: _promptController.text,
              model: "gpt-3.5-turbo"
          );

          final dir = await getApplicationDocumentsDirectory();
          final filename = "${dir.path}/generated_${DateTime.now().millisecondsSinceEpoch}.yaml";
          await File(filename).writeAsString(yaml);

          if (mounted) {
             ScaffoldMessenger.of(context).showSnackBar(const SnackBar(content: Text("Generated successfully! Check Library.")));
          }
      } catch (e) {
          if (mounted) {
             ScaffoldMessenger.of(context).showSnackBar(SnackBar(content: Text("Error: $e")));
          }
      } finally {
          setState(() => _loading = false);
      }
  }

  @override
  Widget build(BuildContext context) {
    return Padding(
        padding: const EdgeInsets.all(16.0),
        child: Column(
            children: [
                TextFormField(
                    controller: _apiKeyController,
                    decoration: const InputDecoration(labelText: "OpenAI API Key"),
                    obscureText: true,
                ),
                const SizedBox(height: 10),
                TextFormField(
                    controller: _promptController,
                    decoration: const InputDecoration(labelText: "Describe your API (e.g. Petstore with 3 endpoints)"),
                    maxLines: 3,
                ),
                const SizedBox(height: 20),
                _loading
                  ? const CircularProgressIndicator()
                  : ElevatedButton.icon(
                      icon: const Icon(Icons.auto_awesome),
                      label: const Text("Generate with AI"),
                      onPressed: _generate,
                    )
            ]
        )
    );
  }
}
