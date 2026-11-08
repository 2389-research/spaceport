"""Specification-tool tests only. These do not run Spaceport or Herdr."""
from __future__ import annotations
import copy
import json
import tempfile
import unittest
from pathlib import Path
import spec


class SpecificationToolsTests(unittest.TestCase):
    def test_duplicate_keys_are_rejected(self):
        with self.assertRaises(ValueError):
            json.loads('{"x":1,"x":2}',object_pairs_hook=spec.unique_object)

    def test_nonfinite_json_is_rejected(self):
        with self.assertRaises(ValueError):
            json.loads('{"x":NaN}',parse_constant=spec.reject_constant)

    def test_duplicate_ids_are_rejected(self):
        with self.assertRaises(ValueError):
            spec.index([{'id':'A'},{'id':'A'}],'fixture')

    def test_cycles_are_rejected(self):
        with self.assertRaises(ValueError):
            spec.validate_graph({'A':{'depends_on':['B']},'B':{'depends_on':['A']}})

    def test_missing_prerequisites_are_rejected(self):
        with self.assertRaises(ValueError):
            spec.validate_graph({'A':{'depends_on':['missing']}})

    def test_acyclic_graph_is_accepted(self):
        spec.validate_graph({'A':{'depends_on':[]},'B':{'depends_on':['A']}})

    def test_packet_rejects_unknown_task(self):
        with self.assertRaises(ValueError):
            spec.task_packet(spec.ROOT,'DOES-NOT-EXIST')

    def test_packet_refuses_silent_truncation(self):
        with self.assertRaises(ValueError):
            spec.task_packet(spec.ROOT,'T00',100)

    def test_packet_contains_required_context(self):
        packet=spec.task_packet(spec.ROOT,'T00')
        self.assertIn('SP-025',packet)
        self.assertIn('input_sha256',packet)
        self.assertIn('No broad code deletion',packet)
        self.assertIn('not_run',packet)

    def test_all_packets_fit_default_budget(self):
        for task in spec.registries(spec.ROOT)['tasks']:
            self.assertLessEqual(len(spec.task_packet(spec.ROOT,task).encode()),65536)

    def test_input_change_changes_hash(self):
        with tempfile.TemporaryDirectory() as d:
            root=Path(d)
            (root/'a.md').write_text('first')
            before=spec.input_hashes(root)
            (root/'a.md').write_text('second')
            self.assertNotEqual(before,spec.input_hashes(root))

    def test_read_rejects_path_escape(self):
        with self.assertRaises(ValueError):
            spec.text(spec.ROOT,'../build_spaceport_spec.py')

    def test_unknown_outcome_cannot_be_retry_safe(self):
        from jsonschema import Draft202012Validator
        schema=spec.load(spec.ROOT/'contracts/control.schema.json')
        example=spec.load(spec.ROOT/'examples/unknown-outcome.json')
        bad=copy.deepcopy(example); bad['retry_safe']=True
        self.assertFalse(Draft202012Validator(schema).is_valid(bad))

    def test_observed_outcome_needs_evidence(self):
        from jsonschema import Draft202012Validator
        schema=spec.load(spec.ROOT/'contracts/control.schema.json')
        example=spec.load(spec.ROOT/'examples/unknown-outcome.json')
        bad=copy.deepcopy(example); bad['state']='observed';bad['evidence']=[]
        self.assertFalse(Draft202012Validator(schema).is_valid(bad))

    def test_investigation_can_complete_with_recorded_open_gap(self):
        task={'id':'T','status':'done','evidence':['documented findings'],
              'gaps':['G'],'must_close_gaps':[]}
        spec.validate_task_completion(task,{'G':{'status':'open'}})

    def test_parity_task_cannot_complete_with_open_gap(self):
        task={'id':'T','status':'done','evidence':['source notes'],
              'gaps':['G'],'must_close_gaps':['G']}
        with self.assertRaises(ValueError):
            spec.validate_task_completion(task,{'G':{'status':'open'}})

    def test_done_task_requires_evidence(self):
        task={'id':'T','status':'done','evidence':[],
              'gaps':[],'must_close_gaps':[]}
        with self.assertRaises(ValueError):
            spec.validate_task_completion(task,{})

    def test_skipped_schema_is_reported_not_run(self):
        report=spec.validate(spec.ROOT,require_schema=False)
        self.assertEqual(report['schema_validation'],'not_run')
        self.assertEqual(report['schema_examples'],0)

    def test_environment_is_not_a_specification_input(self):
        with tempfile.TemporaryDirectory() as d:
            root=Path(d); (root/'.venv').mkdir()
            (root/'.venv'/'installed.json').write_text('not specification JSON')
            (root/'SPEC.md').write_text('fixture')
            self.assertEqual(list(spec.input_hashes(root)), ['SPEC.md'])

    def test_hashing_rejects_symlink_inputs(self):
        with tempfile.TemporaryDirectory() as d:
            root=Path(d); (root/'alias').symlink_to('/etc/passwd')
            with self.assertRaises(ValueError):
                spec.input_hashes(root)

    def test_full_package(self):
        report=spec.validate(spec.ROOT)
        self.assertEqual(report['result'],'passed')
        scenarios=spec.load(spec.ROOT/'plan/acceptance.json')['scenarios']
        executed=sum(item['status'] in {'passed','failed'} for item in scenarios)
        self.assertEqual(report['application_scenarios_executed'],executed)


if __name__=='__main__':
    unittest.main(verbosity=2)
