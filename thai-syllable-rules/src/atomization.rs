use crate::atom::ThaiAtom;
use crate::character_sequence::CharacterSequence;
use crate::construction_scan::ConstructionScan;

impl CharacterSequence<'_> {
    pub(crate) fn thai_atoms(self) -> Vec<ThaiAtom> {
        let characters = self.0;
        let mut atoms: Vec<ThaiAtom> = Vec::with_capacity(characters.len());
        let mut index = 0;

        while index < characters.len() {
            let mut scan = ConstructionScan::new(index, characters[index].character);
            self.scan_onset_and_nucleus(&mut scan, atoms.last());
            let atom_scan = self.complete_scan(&mut scan, atoms.last());
            atoms.push(ThaiAtom::from_scan(
                atom_scan,
                &characters[scan.start..scan.index],
            ));
            index = scan.index;
        }

        atoms
    }
}
